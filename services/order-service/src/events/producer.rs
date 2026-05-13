use anyhow::{Context, Result};
use common_libs::events::{OrderDeliveredEvent, OrderItemEvent, topics};
use rdkafka::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

/// Kafka event producer for order events
#[derive(Clone)]
pub struct OrderEventProducer {
    producer: FutureProducer,
}

impl OrderEventProducer {
    /// Create a new OrderEventProducer
    pub fn new(brokers: &str) -> Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("queue.buffering.max.messages", "10000")
            .set("queue.buffering.max.kbytes", "1048576")
            .set("batch.num.messages", "100")
            .set("compression.type", "lz4")
            .create()
            .context("Failed to create Kafka producer")?;

        Ok(Self { producer })
    }

    /// Publish OrderDeliveredEvent to Kafka
    pub async fn publish_order_delivered(&self, event: OrderDeliveredEvent) -> Result<()> {
        let payload = event.to_json()?;
        let key = event.order_id.clone();

        tracing::info!(
            "Publishing OrderDeliveredEvent for order {} to topic {}",
            event.order_id,
            topics::ORDER_DELIVERED
        );

        let record = FutureRecord::to(topics::ORDER_DELIVERED)
            .key(&key)
            .payload(&payload);

        // Send with timeout
        let delivery_status = self.producer.send(record, Duration::from_secs(5)).await;

        match delivery_status {
            Ok((partition, offset)) => {
                tracing::info!(
                    "Event published successfully: partition={}, offset={}, order_id={}",
                    partition,
                    offset,
                    event.order_id
                );
                Ok(())
            }
            Err((kafka_error, _)) => {
                tracing::error!(
                    "Failed to publish event for order {}: {:?}",
                    event.order_id,
                    kafka_error
                );
                Err(anyhow::anyhow!("Failed to publish event: {}", kafka_error))
            }
        }
    }
}

/// Helper function to convert domain order item to event order item
pub fn convert_order_item_to_event(item: &crate::domain::OrderItem) -> OrderItemEvent {
    OrderItemEvent {
        product_id: item.product_id.clone(),
        product_name: item.product_name.clone(),
        quantity: item.quantity,
        price: item.price,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_order_item_to_event() {
        use crate::domain::OrderItem;
        use chrono::Utc;

        let item = OrderItem {
            id: "item-1".to_string(),
            order_id: "order-1".to_string(),
            product_id: "prod-1".to_string(),
            product_name: "Test Product".to_string(),
            price: 1000,
            quantity: 2,
            subtotal: 2000,
            created_at: Utc::now(),
        };

        let event_item = convert_order_item_to_event(&item);

        assert_eq!(event_item.product_id, item.product_id);
        assert_eq!(event_item.product_name, item.product_name);
        assert_eq!(event_item.quantity, item.quantity);
        assert_eq!(event_item.price, item.price);
    }
}
