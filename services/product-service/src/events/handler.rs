use anyhow::Result;
use common_libs::events::OrderDeliveredEvent;
use std::sync::Arc;

use crate::repository::product::ProductRepository;

/// Event handler for product service
pub struct EventHandler {
    product_repo: Arc<dyn ProductRepository>,
}

impl EventHandler {
    pub fn new(product_repo: Arc<dyn ProductRepository>) -> Self {
        Self { product_repo }
    }

    /// Handle OrderDeliveredEvent by reducing stock for each product
    pub async fn handle_order_delivered(&self, event: OrderDeliveredEvent) -> Result<()> {
        tracing::info!(
            "Handling OrderDeliveredEvent: order_id={}, items_count={}",
            event.order_id,
            event.items.len()
        );

        for item in &event.items {
            // Reduce stock by the ordered quantity
            // quantity_delta is negative to reduce stock
            let quantity_delta = -item.quantity;

            tracing::info!(
                "Reducing stock for product {}: delta={}",
                item.product_id,
                quantity_delta
            );

            match self
                .product_repo
                .update_stock(&item.product_id, quantity_delta)
                .await
            {
                Ok(updated_product) => {
                    tracing::info!(
                        "Stock updated successfully: product_id={}, product_name={}, new_stock={}",
                        updated_product.id,
                        updated_product.name,
                        updated_product.stock_quantity
                    );

                    // Optionally: Check if stock is low and emit a warning
                    if updated_product.stock_quantity < 10 {
                        tracing::warn!(
                            "Low stock alert: product_id={}, product_name={}, stock={}",
                            updated_product.id,
                            updated_product.name,
                            updated_product.stock_quantity
                        );
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to update stock for product {}: {:?}",
                        item.product_id,
                        e
                    );
                    // In a production system, you might want to:
                    // 1. Retry the operation
                    // 2. Send to a dead-letter queue
                    // 3. Emit a metric/alert
                    return Err(anyhow::anyhow!(
                        "Failed to update stock for product {}: {}",
                        item.product_id,
                        e
                    ));
                }
            }
        }

        tracing::info!(
            "Successfully processed OrderDeliveredEvent for order {}",
            event.order_id
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::PaginatedResponse;
    use crate::domain::{
        AppError, CreateProductInput, Product, ProductFilters, ProductResponse, UpdateProductInput,
    };
    use async_trait::async_trait;
    use chrono::Utc;
    use common_libs::events::OrderItemEvent;
    use std::sync::Mutex;

    // Mock repository for testing
    struct MockProductRepository {
        stock_updates: Arc<Mutex<Vec<(String, i32)>>>,
    }

    #[async_trait]
    impl ProductRepository for MockProductRepository {
        async fn create(&self, _input: CreateProductInput) -> Result<Product, AppError> {
            unimplemented!()
        }

        async fn find_by_id(&self, _id: &str) -> Result<Product, AppError> {
            unimplemented!()
        }

        async fn find_by_sku(&self, _sku: &str) -> Result<Product, AppError> {
            unimplemented!()
        }

        async fn find_all(
            &self,
            _filters: ProductFilters,
        ) -> Result<PaginatedResponse<ProductResponse>, AppError> {
            unimplemented!()
        }

        async fn update(&self, _id: &str, _input: UpdateProductInput) -> Result<Product, AppError> {
            unimplemented!()
        }

        async fn delete(&self, _id: &str) -> Result<(), AppError> {
            unimplemented!()
        }

        async fn update_stock(&self, id: &str, quantity_delta: i32) -> Result<Product, AppError> {
            self.stock_updates
                .lock()
                .unwrap()
                .push((id.to_string(), quantity_delta));

            Ok(Product {
                id: id.to_string(),
                sku: "TEST-SKU".to_string(),
                name: "Test Product".to_string(),
                description: Some("Test Description".to_string()),
                price: 1000,
                stock_quantity: 50,
                category: Some("Test".to_string()),
                image_url: None,
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    #[tokio::test]
    async fn test_handle_order_delivered() {
        let stock_updates = Arc::new(Mutex::new(Vec::new()));
        let mock_repo = Arc::new(MockProductRepository {
            stock_updates: stock_updates.clone(),
        });

        let handler = EventHandler::new(mock_repo);

        let event = OrderDeliveredEvent::new(
            "order-123".to_string(),
            "user-456".to_string(),
            vec![
                OrderItemEvent {
                    product_id: "prod-1".to_string(),
                    product_name: "Product 1".to_string(),
                    quantity: 2,
                    price: 1000,
                },
                OrderItemEvent {
                    product_id: "prod-2".to_string(),
                    product_name: "Product 2".to_string(),
                    quantity: 3,
                    price: 2000,
                },
            ],
            Utc::now(),
        );

        handler.handle_order_delivered(event).await.unwrap();

        let updates = stock_updates.lock().unwrap();
        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0], ("prod-1".to_string(), -2));
        assert_eq!(updates[1], ("prod-2".to_string(), -3));
    }
}
