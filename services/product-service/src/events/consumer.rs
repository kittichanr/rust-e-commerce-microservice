use anyhow::{Context, Result};
use common_libs::events::{OrderDeliveredEvent, topics};
use rdkafka::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use std::sync::Arc;
use tokio::task::JoinHandle;

use crate::events::handler::EventHandler;

/// Kafka consumer for product service events
pub struct ProductEventConsumer {
    consumer: StreamConsumer,
    handler: Arc<EventHandler>,
}

impl ProductEventConsumer {
    /// Create a new ProductEventConsumer
    pub fn new(brokers: &str, group_id: &str, handler: Arc<EventHandler>) -> Result<Self> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("group.id", group_id)
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest")
            .set("session.timeout.ms", "30000")
            .set("enable.partition.eof", "false")
            .create()
            .context("Failed to create Kafka consumer")?;

        // Subscribe to topics
        consumer
            .subscribe(&[topics::ORDER_DELIVERED])
            .context("Failed to subscribe to topics")?;

        tracing::info!(
            "Kafka consumer created and subscribed to topics: {}",
            topics::ORDER_DELIVERED
        );

        Ok(Self { consumer, handler })
    }

    /// Start consuming messages in the background
    pub fn start(self) -> JoinHandle<()> {
        tokio::spawn(async move {
            self.consume_loop().await;
        })
    }

    /// Main consumption loop
    async fn consume_loop(self) {
        use tokio_stream::StreamExt;

        tracing::info!("Starting Kafka consumer loop");

        let stream = self.consumer.stream();
        tokio::pin!(stream);

        while let Some(message_result) = stream.next().await {
            match message_result {
                Ok(borrowed_message) => {
                    if let Err(e) = self.process_message(&borrowed_message).await {
                        tracing::error!("Error processing message: {:?}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Kafka consumer error: {:?}", e);
                }
            }
        }

        tracing::warn!("Kafka consumer loop ended");
    }

    /// Process a single Kafka message
    async fn process_message(&self, message: &rdkafka::message::BorrowedMessage<'_>) -> Result<()> {
        let topic = message.topic();
        let partition = message.partition();
        let offset = message.offset();

        tracing::debug!(
            "Received message: topic={}, partition={}, offset={}",
            topic,
            partition,
            offset
        );

        // Extract payload
        let payload = message.payload().context("Message payload is empty")?;

        let payload_str =
            std::str::from_utf8(payload).context("Failed to parse payload as UTF-8")?;

        // Route to appropriate handler based on topic
        match topic {
            topics::ORDER_DELIVERED => {
                let event = OrderDeliveredEvent::from_json(payload_str)
                    .context("Failed to deserialize OrderDeliveredEvent")?;

                tracing::info!(
                    "Processing OrderDeliveredEvent: order_id={}, items={}",
                    event.order_id,
                    event.items.len()
                );

                self.handler.handle_order_delivered(event).await?;
            }
            _ => {
                tracing::warn!("Received message from unknown topic: {}", topic);
            }
        }

        Ok(())
    }
}
