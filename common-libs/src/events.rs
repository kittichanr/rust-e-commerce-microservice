use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Event types for the e-commerce system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum DomainEvent {
    OrderDelivered(OrderDeliveredEvent),
    // Add more events as needed
    // OrderCancelled(OrderCancelledEvent),
    // ProductCreated(ProductCreatedEvent),
}

/// Event published when an order is successfully delivered
/// This triggers stock reduction in the product service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderDeliveredEvent {
    /// Unique event ID
    pub event_id: String,

    /// Order ID that was delivered
    pub order_id: String,

    /// User ID who placed the order
    pub user_id: String,

    /// List of items in the order with quantities
    pub items: Vec<OrderItemEvent>,

    /// Timestamp when the order was delivered
    pub delivered_at: DateTime<Utc>,

    /// Event creation timestamp
    pub event_timestamp: DateTime<Utc>,
}

/// Represents an order item in an event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItemEvent {
    /// Product ID
    pub product_id: String,

    /// Product name (for logging/debugging)
    pub product_name: String,

    /// Quantity ordered
    pub quantity: i32,

    /// Price at time of order (in cents)
    pub price: i64,
}

impl OrderDeliveredEvent {
    /// Create a new OrderDeliveredEvent
    pub fn new(
        order_id: String,
        user_id: String,
        items: Vec<OrderItemEvent>,
        delivered_at: DateTime<Utc>,
    ) -> Self {
        Self {
            event_id: uuid::Uuid::now_v7().to_string(),
            order_id,
            user_id,
            items,
            delivered_at,
            event_timestamp: Utc::now(),
        }
    }

    /// Serialize to JSON for Kafka
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Kafka topic names as constants
pub mod topics {
    pub const ORDER_DELIVERED: &str = "order.delivered";
    pub const ORDER_CANCELLED: &str = "order.cancelled";
    pub const PRODUCT_CREATED: &str = "product.created";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_delivered_event_serialization() {
        let event = OrderDeliveredEvent::new(
            "order-123".to_string(),
            "user-456".to_string(),
            vec![OrderItemEvent {
                product_id: "prod-789".to_string(),
                product_name: "Test Product".to_string(),
                quantity: 2,
                price: 1000,
            }],
            Utc::now(),
        );

        let json = event.to_json().unwrap();
        let deserialized = OrderDeliveredEvent::from_json(&json).unwrap();

        assert_eq!(event.order_id, deserialized.order_id);
        assert_eq!(event.user_id, deserialized.user_id);
        assert_eq!(event.items.len(), deserialized.items.len());
    }
}
