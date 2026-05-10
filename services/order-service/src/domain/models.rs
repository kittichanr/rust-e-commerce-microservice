use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

// ========== Order Status ==========

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR(50)", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    Cart,
    Checkout,
    PaymentPending,
    PaymentFailed,
    Confirmed,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
}

impl OrderStatus {
    pub fn can_transition_to(&self, new_status: &OrderStatus) -> bool {
        use OrderStatus::*;
        
        matches!(
            (self, new_status),
            (Cart, Checkout)
                | (Checkout, PaymentPending)
                | (PaymentPending, Confirmed)
                | (PaymentPending, PaymentFailed)
                | (Confirmed, Processing)
                | (Processing, Shipped)
                | (Shipped, Delivered)
                | (Cart, Cancelled)
                | (Checkout, Cancelled)
                | (PaymentPending, Cancelled)
                | (Confirmed, Cancelled)
        )
    }
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            OrderStatus::Cart => "CART",
            OrderStatus::Checkout => "CHECKOUT",
            OrderStatus::PaymentPending => "PAYMENT_PENDING",
            OrderStatus::PaymentFailed => "PAYMENT_FAILED",
            OrderStatus::Confirmed => "CONFIRMED",
            OrderStatus::Processing => "PROCESSING",
            OrderStatus::Shipped => "SHIPPED",
            OrderStatus::Delivered => "DELIVERED",
            OrderStatus::Cancelled => "CANCELLED",
        };
        write!(f, "{}", s)
    }
}

// ========== Order Item ==========

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderItem {
    pub id: String,
    pub order_id: String,
    pub product_id: String,
    pub product_name: String,
    pub price: i64,        // Price in cents at time of order
    pub quantity: i32,
    pub subtotal: i64,     // price * quantity
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateOrderItemInput {
    #[validate(length(min = 1))]
    pub product_id: String,
    
    #[validate(length(min = 1))]
    pub product_name: String,
    
    #[validate(range(min = 0))]
    pub price: i64,
    
    #[validate(range(min = 1))]
    pub quantity: i32,
}

// ========== Order Models ==========

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub subtotal: i64,         // Sum of all items
    pub tax: i64,              // Tax amount in cents
    pub shipping_fee: i64,     // Shipping fee in cents
    pub total: i64,            // subtotal + tax + shipping_fee
    pub status: OrderStatus,
    pub shipping_address: Option<String>,
    pub billing_address: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateOrderInput {
    #[validate(length(min = 1))]
    pub user_id: String,
    
    #[validate(length(min = 1))]
    pub items: Vec<CreateOrderItemInput>,
    
    pub shipping_address: Option<String>,
    pub billing_address: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderWithItems {
    pub order: Order,
    pub items: Vec<OrderItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub id: String,
    pub user_id: String,
    pub items: Vec<OrderItemResponse>,
    pub subtotal: i64,
    pub tax: i64,
    pub shipping_fee: i64,
    pub total: i64,
    pub status: OrderStatus,
    pub shipping_address: Option<String>,
    pub billing_address: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItemResponse {
    pub product_id: String,
    pub product_name: String,
    pub price: i64,
    pub quantity: i32,
    pub subtotal: i64,
}

impl From<OrderWithItems> for OrderResponse {
    fn from(order_with_items: OrderWithItems) -> Self {
        Self {
            id: order_with_items.order.id,
            user_id: order_with_items.order.user_id,
            items: order_with_items
                .items
                .into_iter()
                .map(|item| OrderItemResponse {
                    product_id: item.product_id,
                    product_name: item.product_name,
                    price: item.price,
                    quantity: item.quantity,
                    subtotal: item.subtotal,
                })
                .collect(),
            subtotal: order_with_items.order.subtotal,
            tax: order_with_items.order.tax,
            shipping_fee: order_with_items.order.shipping_fee,
            total: order_with_items.order.total,
            status: order_with_items.order.status,
            shipping_address: order_with_items.order.shipping_address,
            billing_address: order_with_items.order.billing_address,
            notes: order_with_items.order.notes,
            created_at: order_with_items.order.created_at,
            updated_at: order_with_items.order.updated_at,
        }
    }
}

// ========== Pagination & Filters ==========

#[derive(Debug, Clone, Deserialize)]
pub struct OrderFilters {
    pub user_id: Option<String>,
    pub status: Option<OrderStatus>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

impl Default for OrderFilters {
    fn default() -> Self {
        Self {
            user_id: None,
            status: None,
            page: Some(1),
            per_page: Some(20),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}
