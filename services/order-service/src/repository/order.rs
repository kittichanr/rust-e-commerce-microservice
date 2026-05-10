use async_trait::async_trait;
use sqlx::{MySql, Pool};
use uuid::Uuid;

use crate::domain::{
    AppError, CreateOrderInput, Order, OrderFilters, OrderItem, OrderResponse, OrderStatus,
    OrderWithItems, PaginatedResponse,
};

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn create(&self, input: CreateOrderInput) -> Result<OrderWithItems, AppError>;
    async fn find_by_id(&self, id: &str) -> Result<OrderWithItems, AppError>;
    async fn find_all(
        &self,
        filters: OrderFilters,
    ) -> Result<PaginatedResponse<OrderResponse>, AppError>;
    async fn update_status(
        &self,
        id: &str,
        status: OrderStatus,
    ) -> Result<OrderWithItems, AppError>;
    async fn cancel(&self, id: &str) -> Result<(), AppError>;
}

#[derive(Clone)]
pub struct MySqlOrderRepository {
    pool: Pool<MySql>,
}

impl MySqlOrderRepository {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }

    async fn find_order_items(&self, order_id: &str) -> Result<Vec<OrderItem>, AppError> {
        let items = sqlx::query_as::<_, OrderItem>(
            "SELECT * FROM order_items WHERE order_id = ? ORDER BY created_at ASC",
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    fn calculate_order_totals(items: &[crate::domain::CreateOrderItemInput]) -> (i64, i64, i64, i64) {
        let subtotal: i64 = items.iter().map(|item| item.price * item.quantity as i64).sum();
        
        // Simple tax calculation: 10% of subtotal
        let tax = (subtotal as f64 * 0.10) as i64;
        
        // Simple shipping fee: $10 (1000 cents) flat rate
        let shipping_fee = 1000i64;
        
        let total = subtotal + tax + shipping_fee;
        
        (subtotal, tax, shipping_fee, total)
    }
}

#[async_trait]
impl OrderRepository for MySqlOrderRepository {
    async fn create(&self, input: CreateOrderInput) -> Result<OrderWithItems, AppError> {
        // Validate items exist
        if input.items.is_empty() {
            return Err(AppError::Validation("Order must contain at least one item".to_string()));
        }

        // Calculate totals
        let (subtotal, tax, shipping_fee, total) = Self::calculate_order_totals(&input.items);

        // Start transaction
        let mut tx = self.pool.begin().await?;

        // Create order
        let order_id = Uuid::now_v7().to_string();

        sqlx::query(
            r#"
            INSERT INTO orders (
                id, user_id, subtotal, tax, shipping_fee, total, status,
                shipping_address, billing_address, notes
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&order_id)
        .bind(&input.user_id)
        .bind(subtotal)
        .bind(tax)
        .bind(shipping_fee)
        .bind(total)
        .bind("CART")
        .bind(&input.shipping_address)
        .bind(&input.billing_address)
        .bind(&input.notes)
        .execute(&mut *tx)
        .await?;

        // Create order items
        for item in &input.items {
            let item_id = Uuid::now_v7().to_string();
            let item_subtotal = item.price * item.quantity as i64;

            sqlx::query(
                r#"
                INSERT INTO order_items (
                    id, order_id, product_id, product_name, price, quantity, subtotal
                )
                VALUES (?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&item_id)
            .bind(&order_id)
            .bind(&item.product_id)
            .bind(&item.product_name)
            .bind(item.price)
            .bind(item.quantity)
            .bind(item_subtotal)
            .execute(&mut *tx)
            .await?;
        }

        // Commit transaction
        tx.commit().await?;

        // Return created order with items
        self.find_by_id(&order_id).await
    }

    async fn find_by_id(&self, id: &str) -> Result<OrderWithItems, AppError> {
        let order = sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Order with id '{}' not found", id)))?;

        let items = self.find_order_items(id).await?;

        Ok(OrderWithItems { order, items })
    }

    async fn find_all(
        &self,
        filters: OrderFilters,
    ) -> Result<PaginatedResponse<OrderResponse>, AppError> {
        let page = filters.page.unwrap_or(1).max(1);
        let per_page = filters.per_page.unwrap_or(20).min(100);
        let offset = (page - 1) * per_page;

        // Build dynamic query
        let mut query = String::from("SELECT * FROM orders WHERE 1=1");
        let mut count_query = String::from("SELECT COUNT(*) as count FROM orders WHERE 1=1");

        // Add filters
        if filters.user_id.is_some() {
            query.push_str(" AND user_id = ?");
            count_query.push_str(" AND user_id = ?");
        }
        if let Some(status) = filters.status {
            let status_str = status.to_string();
            query.push_str(&format!(" AND status = '{}'", status_str));
            count_query.push_str(&format!(" AND status = '{}'", status_str));
        }

        query.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

        // Execute count query
        let mut count_q = sqlx::query_scalar::<_, i64>(&count_query);
        if let Some(ref user_id) = filters.user_id {
            count_q = count_q.bind(user_id);
        }

        let total = count_q.fetch_one(&self.pool).await? as u64;

        // Execute main query
        let mut main_q = sqlx::query_as::<_, Order>(&query);
        if let Some(ref user_id) = filters.user_id {
            main_q = main_q.bind(user_id);
        }
        main_q = main_q.bind(per_page as i64).bind(offset as i64);

        let orders = main_q.fetch_all(&self.pool).await?;

        // Fetch items for each order
        let mut order_responses = Vec::new();
        for order in orders {
            let items = self.find_order_items(&order.id).await?;
            let order_with_items = OrderWithItems { order, items };
            order_responses.push(OrderResponse::from(order_with_items));
        }

        let total_pages = (total + per_page - 1) / per_page;

        Ok(PaginatedResponse {
            data: order_responses,
            total,
            page,
            per_page,
            total_pages,
        })
    }

    async fn update_status(
        &self,
        id: &str,
        new_status: OrderStatus,
    ) -> Result<OrderWithItems, AppError> {
        // Get current order
        let order_with_items = self.find_by_id(id).await?;
        let current_status = order_with_items.order.status;

        // Check if transition is valid
        if !current_status.can_transition_to(&new_status) {
            return Err(AppError::InvalidStatusTransition(format!(
                "Cannot transition from {} to {}",
                current_status, new_status
            )));
        }

        // Update status
        let new_status_str = new_status.to_string();
        sqlx::query("UPDATE orders SET status = ? WHERE id = ?")
            .bind(&new_status_str)
            .bind(id)
            .execute(&self.pool)
            .await?;

        // Return updated order
        self.find_by_id(id).await
    }

    async fn cancel(&self, id: &str) -> Result<(), AppError> {
        // Get current order
        let order_with_items = self.find_by_id(id).await?;
        
        // Check if order can be cancelled
        if !order_with_items
            .order
            .status
            .can_transition_to(&OrderStatus::Cancelled)
        {
            return Err(AppError::InvalidStatusTransition(format!(
                "Cannot cancel order in {} status",
                order_with_items.order.status
            )));
        }

        // Update to cancelled
        sqlx::query("UPDATE orders SET status = 'CANCELLED' WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
