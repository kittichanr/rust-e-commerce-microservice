use std::sync::Arc;

use common_libs::proto::order::order_server::Order as OrderService;
use common_libs::proto::order::{
    CancelOrderRequest, CancelOrderResponse, CreateOrderRequest, CreateOrderResponse,
    GetOrderRequest, GetOrderResponse, ListOrdersRequest, ListOrdersResponse, OrderInfo,
    OrderItem as ProtoOrderItem, UpdateOrderStatusRequest, UpdateOrderStatusResponse,
};
use common_libs::proto::product::GetProductRequest;
use common_libs::proto::product::product_client::ProductClient;
use tonic::transport::Channel;
use tonic::{Request, Response, Status};

use crate::domain::{CreateOrderInput, CreateOrderItemInput};
use crate::repository::order::OrderRepository;

pub struct MyOrderService {
    order_repo: Arc<dyn OrderRepository>,
    product_client: ProductClient<Channel>,
}

impl MyOrderService {
    pub fn new(
        order_repo: Arc<dyn OrderRepository>,
        product_client: ProductClient<Channel>,
    ) -> Self {
        Self {
            order_repo,
            product_client,
        }
    }
}

#[tonic::async_trait]
impl OrderService for MyOrderService {
    async fn create_order(
        &self,
        request: Request<CreateOrderRequest>,
    ) -> Result<Response<CreateOrderResponse>, Status> {
        let req = request.into_inner();

        // Validate request has items
        if req.items.is_empty() {
            return Err(Status::invalid_argument(
                "Order must contain at least one item",
            ));
        }

        // Validate each item with product service
        let mut validated_items = Vec::new();

        for proto_item in &req.items {
            // Clone the client for each request (client is cheap to clone)
            let mut product_client = self.product_client.clone();

            // 1. Validate product exists and fetch details
            let product_response = product_client
                .get_product(GetProductRequest {
                    product_id: proto_item.product_id.clone(),
                })
                .await
                .map_err(|e| {
                    tracing::warn!("Failed to fetch product {}: {}", proto_item.product_id, e);
                    Status::not_found(format!("Product '{}' not found", proto_item.product_id))
                })?;

            let product = product_response.into_inner().product.ok_or_else(|| {
                Status::not_found(format!("Product '{}' not found", proto_item.product_id))
            })?;

            // 2. Check product is active
            if !product.is_active {
                return Err(Status::failed_precondition(format!(
                    "Product '{}' is no longer available",
                    product.name
                )));
            }

            // 3. Validate stock availability
            if product.stock_quantity < proto_item.quantity {
                return Err(Status::failed_precondition(format!(
                    "Insufficient stock for product '{}'. Available: {}, Requested: {}",
                    product.name, product.stock_quantity, proto_item.quantity
                )));
            }

            // 4. Use authoritative price from product service (don't trust client)
            // Note: In a real system, you might want to allow slight price differences
            // and warn the user, but here we enforce exact match
            if proto_item.price != product.price {
                return Err(Status::invalid_argument(format!(
                    "Price mismatch for product '{}'. Expected: {}, Received: {}. Please refresh your cart.",
                    product.name, product.price, proto_item.price
                )));
            }

            // Validate quantity is positive
            if proto_item.quantity <= 0 {
                return Err(Status::invalid_argument(format!(
                    "Invalid quantity for product '{}': must be greater than 0",
                    product.name
                )));
            }

            // Create validated item with authoritative data
            validated_items.push(CreateOrderItemInput {
                product_id: product.product_id,
                product_name: product.name,
                price: product.price,
                quantity: proto_item.quantity,
            });
        }

        // Create order input
        let order_input = CreateOrderInput {
            user_id: req.user_id,
            items: validated_items,
            shipping_address: req.shipping_address,
            billing_address: req.billing_address,
            notes: req.notes,
        };

        // Create order in database
        let order_with_items = self.order_repo.create(order_input).await.map_err(|e| {
            tracing::error!("Failed to create order: {:?}", e);
            Status::internal(format!("Failed to create order: {}", e))
        })?;

        tracing::info!("Order created successfully: {}", order_with_items.order.id);

        // Convert to proto response
        let order_info = convert_to_order_info(order_with_items);

        Ok(Response::new(CreateOrderResponse {
            success: true,
            message: "Order created successfully".to_string(),
            order: Some(order_info),
        }))
    }

    async fn get_order(
        &self,
        request: Request<GetOrderRequest>,
    ) -> Result<Response<GetOrderResponse>, Status> {
        let req: GetOrderRequest = request.into_inner();

        // Validate order_id is provided
        if req.order_id.is_empty() {
            return Err(Status::invalid_argument("Order ID is required"));
        }

        // Fetch order from repository
        let order_with_items =
            self.order_repo
                .find_by_id(&req.order_id)
                .await
                .map_err(|e| match e {
                    crate::domain::AppError::NotFound(msg) => {
                        tracing::warn!("Order not found: {}", msg);
                        Status::not_found(msg)
                    }
                    _ => {
                        tracing::error!("Failed to fetch order: {:?}", e);
                        Status::internal(format!("Failed to fetch order: {}", e))
                    }
                })?;

        tracing::info!(
            "Order retrieved successfully: {}",
            order_with_items.order.id
        );

        // Convert to proto response
        let order_info = convert_to_order_info(order_with_items);

        Ok(Response::new(GetOrderResponse {
            success: true,
            message: "Order retrieved successfully".to_string(),
            order: Some(order_info),
        }))
    }

    async fn update_order_status(
        &self,
        request: Request<UpdateOrderStatusRequest>,
    ) -> Result<Response<UpdateOrderStatusResponse>, Status> {
        let req = request.into_inner();

        // Validate order_id is provided
        if req.order_id.is_empty() {
            return Err(Status::invalid_argument("Order ID is required"));
        }

        // Validate and convert proto status to domain status
        if req.status == 0 {
            return Err(Status::invalid_argument(
                "Invalid status: ORDER_STATUS_UNSPECIFIED is not allowed",
            ));
        }

        let new_status = convert_proto_status_to_domain(req.status).ok_or_else(|| {
            Status::invalid_argument(format!("Invalid order status: {}", req.status))
        })?;

        // Update order status in repository
        let order_with_items = self
            .order_repo
            .update_status(&req.order_id, new_status)
            .await
            .map_err(|e| match e {
                crate::domain::AppError::NotFound(msg) => {
                    tracing::warn!("Order not found: {}", msg);
                    Status::not_found(msg)
                }
                crate::domain::AppError::InvalidStatusTransition(msg) => {
                    tracing::warn!("Invalid status transition: {}", msg);
                    Status::failed_precondition(msg)
                }
                _ => {
                    tracing::error!("Failed to update order status: {:?}", e);
                    Status::internal(format!("Failed to update order status: {}", e))
                }
            })?;

        tracing::info!(
            "Order status updated successfully: {} -> {}",
            order_with_items.order.id,
            order_with_items.order.status
        );

        // Convert to proto response
        let order_info = convert_to_order_info(order_with_items);

        Ok(Response::new(UpdateOrderStatusResponse {
            success: true,
            message: "Order status updated successfully".to_string(),
            order: Some(order_info),
        }))
    }

    async fn cancel_order(
        &self,
        request: Request<CancelOrderRequest>,
    ) -> Result<Response<CancelOrderResponse>, Status> {
        let req = request.into_inner();

        // Validate order_id is provided
        if req.order_id.is_empty() {
            return Err(Status::invalid_argument("Order ID is required"));
        }

        // Extract and clean cancellation reason
        let reason = req.reason.filter(|r| !r.trim().is_empty());

        // Log cancellation reason if provided
        if let Some(ref r) = reason {
            tracing::info!(
                "Order {} cancellation requested with reason: {}",
                req.order_id,
                r
            );
        }

        // Cancel order in repository with reason
        self.order_repo
            .cancel(&req.order_id, reason)
            .await
            .map_err(|e| match e {
                crate::domain::AppError::NotFound(msg) => {
                    tracing::warn!("Order not found: {}", msg);
                    Status::not_found(msg)
                }
                crate::domain::AppError::InvalidStatusTransition(msg) => {
                    tracing::warn!("Cannot cancel order: {}", msg);
                    Status::failed_precondition(msg)
                }
                _ => {
                    tracing::error!("Failed to cancel order: {:?}", e);
                    Status::internal(format!("Failed to cancel order: {}", e))
                }
            })?;

        tracing::info!("Order cancelled successfully: {}", req.order_id);

        Ok(Response::new(CancelOrderResponse {
            success: true,
            message: "Order cancelled successfully".to_string(),
        }))
    }

    async fn list_orders(
        &self,
        request: Request<ListOrdersRequest>,
    ) -> Result<Response<ListOrdersResponse>, Status> {
        let req = request.into_inner();

        // Convert proto OrderStatus to domain OrderStatus if provided
        let status = if let Some(proto_status) = req.status {
            if proto_status != 0 {
                // 0 is ORDER_STATUS_UNSPECIFIED, meaning no filter
                Some(convert_proto_status_to_domain(proto_status).ok_or_else(|| {
                    Status::invalid_argument(format!("Invalid order status: {}", proto_status))
                })?)
            } else {
                None
            }
        } else {
            None
        };

        // Build filters
        let filters = crate::domain::OrderFilters {
            user_id: req.user_id.filter(|id| !id.is_empty()),
            status,
            page: req.page,
            per_page: req.per_page,
        };

        // Fetch orders from repository
        let paginated_result = self.order_repo.find_all(filters).await.map_err(|e| {
            tracing::error!("Failed to list orders: {:?}", e);
            Status::internal(format!("Failed to list orders: {}", e))
        })?;

        tracing::info!(
            "Listed {} orders (page {}/{})",
            paginated_result.data.len(),
            paginated_result.page,
            paginated_result.total_pages
        );

        // Convert OrderResponse to OrderInfo (proto format)
        let order_infos: Vec<OrderInfo> = paginated_result
            .data
            .into_iter()
            .map(convert_order_response_to_order_info)
            .collect();

        Ok(Response::new(ListOrdersResponse {
            success: true,
            message: format!("Found {} orders", order_infos.len()),
            orders: order_infos,
            total: paginated_result.total,
            page: paginated_result.page,
            per_page: paginated_result.per_page,
            total_pages: paginated_result.total_pages,
        }))
    }
}

// Helper function to convert domain model to proto
fn convert_to_order_info(order_with_items: crate::domain::OrderWithItems) -> OrderInfo {
    let proto_items: Vec<ProtoOrderItem> = order_with_items
        .items
        .into_iter()
        .map(|item| ProtoOrderItem {
            product_id: item.product_id,
            product_name: item.product_name,
            price: item.price,
            quantity: item.quantity,
            subtotal: item.subtotal,
        })
        .collect();

    // Convert OrderStatus enum
    let status = match order_with_items.order.status {
        crate::domain::OrderStatus::Cart => 1,
        crate::domain::OrderStatus::Checkout => 2,
        crate::domain::OrderStatus::PaymentPending => 3,
        crate::domain::OrderStatus::PaymentFailed => 4,
        crate::domain::OrderStatus::Confirmed => 5,
        crate::domain::OrderStatus::Processing => 6,
        crate::domain::OrderStatus::Shipped => 7,
        crate::domain::OrderStatus::Delivered => 8,
        crate::domain::OrderStatus::Cancelled => 9,
    };

    OrderInfo {
        order_id: order_with_items.order.id,
        user_id: order_with_items.order.user_id,
        items: proto_items,
        subtotal: order_with_items.order.subtotal,
        tax: order_with_items.order.tax,
        shipping_fee: order_with_items.order.shipping_fee,
        total: order_with_items.order.total,
        status,
        shipping_address: order_with_items.order.shipping_address,
        billing_address: order_with_items.order.billing_address,
        notes: order_with_items.order.notes,
        created_at: order_with_items.order.created_at.timestamp(),
        updated_at: order_with_items.order.updated_at.timestamp(),
        cancellation_reason: order_with_items.order.cancellation_reason,
    }
}

// Helper function to convert OrderResponse to OrderInfo (for list_orders)
fn convert_order_response_to_order_info(order_response: crate::domain::OrderResponse) -> OrderInfo {
    let proto_items: Vec<ProtoOrderItem> = order_response
        .items
        .into_iter()
        .map(|item| ProtoOrderItem {
            product_id: item.product_id,
            product_name: item.product_name,
            price: item.price,
            quantity: item.quantity,
            subtotal: item.subtotal,
        })
        .collect();

    // Convert OrderStatus enum
    let status = match order_response.status {
        crate::domain::OrderStatus::Cart => 1,
        crate::domain::OrderStatus::Checkout => 2,
        crate::domain::OrderStatus::PaymentPending => 3,
        crate::domain::OrderStatus::PaymentFailed => 4,
        crate::domain::OrderStatus::Confirmed => 5,
        crate::domain::OrderStatus::Processing => 6,
        crate::domain::OrderStatus::Shipped => 7,
        crate::domain::OrderStatus::Delivered => 8,
        crate::domain::OrderStatus::Cancelled => 9,
    };

    OrderInfo {
        order_id: order_response.id,
        user_id: order_response.user_id,
        items: proto_items,
        subtotal: order_response.subtotal,
        tax: order_response.tax,
        shipping_fee: order_response.shipping_fee,
        total: order_response.total,
        status,
        shipping_address: order_response.shipping_address,
        billing_address: order_response.billing_address,
        notes: order_response.notes,
        created_at: order_response.created_at.timestamp(),
        updated_at: order_response.updated_at.timestamp(),
        cancellation_reason: order_response.cancellation_reason,
    }
}

// Helper function to convert proto status enum (i32) to domain OrderStatus
fn convert_proto_status_to_domain(proto_status: i32) -> Option<crate::domain::OrderStatus> {
    match proto_status {
        1 => Some(crate::domain::OrderStatus::Cart),
        2 => Some(crate::domain::OrderStatus::Checkout),
        3 => Some(crate::domain::OrderStatus::PaymentPending),
        4 => Some(crate::domain::OrderStatus::PaymentFailed),
        5 => Some(crate::domain::OrderStatus::Confirmed),
        6 => Some(crate::domain::OrderStatus::Processing),
        7 => Some(crate::domain::OrderStatus::Shipped),
        8 => Some(crate::domain::OrderStatus::Delivered),
        9 => Some(crate::domain::OrderStatus::Cancelled),
        _ => None,
    }
}
