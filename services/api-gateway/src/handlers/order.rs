use crate::{error::GatewayError, middleware::Claims, AppState};
use actix_web::{web, HttpResponse};
use common_libs::proto::order::{
    order_client::OrderClient, CreateOrderRequest, GetOrderRequest,
    ListOrdersRequest, OrderItem as ProtoOrderItem, OrderStatus, UpdateOrderStatusRequest,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct OrderItemInput {
    pub product_id: String,
    pub product_name: String,
    pub quantity: i32,
    pub price: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderInput {
    pub items: Vec<OrderItemInput>,
    pub shipping_address: Option<String>,
    pub billing_address: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatusInput {
    pub status: i32, // OrderStatus enum as i32
}

#[derive(Debug, Serialize)]
pub struct OrderItemResponse {
    pub product_id: String,
    pub product_name: String,
    pub quantity: i32,
    pub price: i64,
    pub subtotal: i64,
}

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub user_id: String,
    pub items: Vec<OrderItemResponse>,
    pub subtotal: i64,
    pub tax: i64,
    pub shipping_fee: i64,
    pub total: i64,
    pub status: i32,
    pub shipping_address: Option<String>,
    pub billing_address: Option<String>,
    pub notes: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize)]
pub struct OrderListResponse {
    pub orders: Vec<OrderResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

pub async fn create_order(
    state: web::Data<AppState>,
    input: web::Json<CreateOrderInput>,
    claims: Claims,
) -> Result<HttpResponse, GatewayError> {
    let mut client = OrderClient::connect(state.config.services.order_service_url.clone())
        .await
        .map_err(|e| GatewayError::ServiceUnavailable(format!("Order service: {}", e)))?;

    let items: Vec<ProtoOrderItem> = input
        .items
        .iter()
        .map(|item| ProtoOrderItem {
            product_id: item.product_id.clone(),
            product_name: item.product_name.clone(),
            price: item.price,
            quantity: item.quantity,
            subtotal: item.price * item.quantity as i64,
        })
        .collect();

    let request = tonic::Request::new(CreateOrderRequest {
        user_id: claims.sub,
        items,
        shipping_address: input.shipping_address.clone(),
        billing_address: input.billing_address.clone(),
        notes: input.notes.clone(),
    });

    let response = client.create_order(request).await?.into_inner();
    let order = response.order.ok_or(GatewayError::Internal("Order not returned".to_string()))?;

    let items_response: Vec<OrderItemResponse> = order
        .items
        .into_iter()
        .map(|item| OrderItemResponse {
            product_id: item.product_id,
            product_name: item.product_name,
            quantity: item.quantity,
            price: item.price,
            subtotal: item.subtotal,
        })
        .collect();

    Ok(HttpResponse::Created().json(OrderResponse {
        order_id: order.order_id,
        user_id: order.user_id,
        items: items_response,
        subtotal: order.subtotal,
        tax: order.tax,
        shipping_fee: order.shipping_fee,
        total: order.total,
        status: order.status,
        shipping_address: order.shipping_address,
        billing_address: order.billing_address,
        notes: order.notes,
        created_at: order.created_at,
        updated_at: order.updated_at,
    }))
}

pub async fn get_order(
    state: web::Data<AppState>,
    order_id: web::Path<String>,
    _claims: Claims,
) -> Result<HttpResponse, GatewayError> {
    let mut client = OrderClient::connect(state.config.services.order_service_url.clone())
        .await
        .map_err(|e| GatewayError::ServiceUnavailable(format!("Order service: {}", e)))?;

    let request = tonic::Request::new(GetOrderRequest {
        order_id: order_id.into_inner(),
    });

    let response = client.get_order(request).await?.into_inner();
    let order = response.order.ok_or(GatewayError::NotFound)?;

    let items_response: Vec<OrderItemResponse> = order
        .items
        .into_iter()
        .map(|item| OrderItemResponse {
            product_id: item.product_id,
            product_name: item.product_name,
            quantity: item.quantity,
            price: item.price,
            subtotal: item.subtotal,
        })
        .collect();

    Ok(HttpResponse::Ok().json(OrderResponse {
        order_id: order.order_id,
        user_id: order.user_id,
        items: items_response,
        subtotal: order.subtotal,
        tax: order.tax,
        shipping_fee: order.shipping_fee,
        total: order.total,
        status: order.status,
        shipping_address: order.shipping_address,
        billing_address: order.billing_address,
        notes: order.notes,
        created_at: order.created_at,
        updated_at: order.updated_at,
    }))
}

pub async fn list_orders(
    state: web::Data<AppState>,
    claims: Claims,
) -> Result<HttpResponse, GatewayError> {
    let mut client = OrderClient::connect(state.config.services.order_service_url.clone())
        .await
        .map_err(|e| GatewayError::ServiceUnavailable(format!("Order service: {}", e)))?;

    let request = tonic::Request::new(ListOrdersRequest {
        user_id: Some(claims.sub),
        status: None,
        page: None,
        per_page: None,
    });

    let response = client.list_orders(request).await?.into_inner();

    let orders: Vec<OrderResponse> = response
        .orders
        .into_iter()
        .map(|order| {
            let items: Vec<OrderItemResponse> = order
                .items
                .into_iter()
                .map(|item| OrderItemResponse {
                    product_id: item.product_id,
                    product_name: item.product_name,
                    quantity: item.quantity,
                    price: item.price,
                    subtotal: item.subtotal,
                })
                .collect();

            OrderResponse {
                order_id: order.order_id,
                user_id: order.user_id,
                items,
                subtotal: order.subtotal,
                tax: order.tax,
                shipping_fee: order.shipping_fee,
                total: order.total,
                status: order.status,
                shipping_address: order.shipping_address,
                billing_address: order.billing_address,
                notes: order.notes,
                created_at: order.created_at,
                updated_at: order.updated_at,
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(OrderListResponse {
        orders,
        total: response.total,
        page: response.page,
        per_page: response.per_page,
        total_pages: response.total_pages,
    }))
}

pub async fn update_order_status(
    state: web::Data<AppState>,
    order_id: web::Path<String>,
    input: web::Json<UpdateOrderStatusInput>,
    _claims: Claims,
) -> Result<HttpResponse, GatewayError> {
    let mut client = OrderClient::connect(state.config.services.order_service_url.clone())
        .await
        .map_err(|e| GatewayError::ServiceUnavailable(format!("Order service: {}", e)))?;

    let status = OrderStatus::try_from(input.status)
        .map_err(|_| GatewayError::BadRequest("Invalid order status".to_string()))?;

    let request = tonic::Request::new(UpdateOrderStatusRequest {
        order_id: order_id.into_inner(),
        status: status as i32,
    });

    let response = client.update_order_status(request).await?.into_inner();
    let order = response.order.ok_or(GatewayError::NotFound)?;

    let items_response: Vec<OrderItemResponse> = order
        .items
        .into_iter()
        .map(|item| OrderItemResponse {
            product_id: item.product_id,
            product_name: item.product_name,
            quantity: item.quantity,
            price: item.price,
            subtotal: item.subtotal,
        })
        .collect();

    Ok(HttpResponse::Ok().json(OrderResponse {
        order_id: order.order_id,
        user_id: order.user_id,
        items: items_response,
        subtotal: order.subtotal,
        tax: order.tax,
        shipping_fee: order.shipping_fee,
        total: order.total,
        status: order.status,
        shipping_address: order.shipping_address,
        billing_address: order.billing_address,
        notes: order.notes,
        created_at: order.created_at,
        updated_at: order.updated_at,
    }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/orders")
            .route("", web::post().to(create_order))
            .route("", web::get().to(list_orders))
            .route("/{id}", web::get().to(get_order))
            .route("/{id}/status", web::put().to(update_order_status)),
    );
}
