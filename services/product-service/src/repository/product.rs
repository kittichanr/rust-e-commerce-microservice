use async_trait::async_trait;
use sqlx::{MySql, Pool};
use uuid::Uuid;

use crate::domain::{
    AppError, CreateProductInput, PaginatedResponse, Product, ProductFilters, ProductResponse,
    UpdateProductInput,
};

#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn create(&self, input: CreateProductInput) -> Result<Product, AppError>;
    async fn find_by_id(&self, id: &str) -> Result<Product, AppError>;
    async fn find_by_sku(&self, sku: &str) -> Result<Product, AppError>;
    async fn find_all(
        &self,
        filters: ProductFilters,
    ) -> Result<PaginatedResponse<ProductResponse>, AppError>;
    async fn update(&self, id: &str, input: UpdateProductInput) -> Result<Product, AppError>;
    async fn delete(&self, id: &str) -> Result<(), AppError>;
    async fn update_stock(&self, id: &str, quantity_delta: i32) -> Result<Product, AppError>;
}

#[derive(Clone)]

pub struct MySqlProductRepository {
    pool: Pool<MySql>,
}

impl MySqlProductRepository {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductRepository for MySqlProductRepository {
    async fn create(&self, input: CreateProductInput) -> Result<Product, AppError> {
        let id = Uuid::now_v7().to_string();

        let result = sqlx::query(
            r#"
            INSERT INTO products (
                id, sku, name, description, price, stock_quantity,
                category, image_url, is_active
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&input.sku)
        .bind(&input.name)
        .bind(&input.description)
        .bind(input.price)
        .bind(input.stock_quantity)
        .bind(&input.category)
        .bind(&input.image_url)
        .bind(input.is_active.unwrap_or(true))
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => self.find_by_id(&id).await,
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                Err(AppError::Conflict(format!("Product with SKU '{}' already exists", input.sku)))
            }
            Err(e) => Err(AppError::Database(e)),
        }
    }

    async fn find_by_id(&self, id: &str) -> Result<Product, AppError> {
        let product = sqlx::query_as::<_, Product>(
            "SELECT * FROM products WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        product.ok_or_else(|| AppError::NotFound(format!("Product with id '{}' not found", id)))
    }

    async fn find_by_sku(&self, sku: &str) -> Result<Product, AppError> {
        let product = sqlx::query_as::<_, Product>(
            "SELECT * FROM products WHERE sku = ?"
        )
        .bind(sku)
        .fetch_optional(&self.pool)
        .await?;

        product.ok_or_else(|| AppError::NotFound(format!("Product with SKU '{}' not found", sku)))
    }

    async fn find_all(
        &self,
        filters: ProductFilters,
    ) -> Result<PaginatedResponse<ProductResponse>, AppError> {
        let page = filters.page.unwrap_or(1).max(1);
        let per_page = filters.per_page.unwrap_or(20).min(100);
        let offset = (page - 1) * per_page;

        // Build dynamic query
        let mut query = String::from("SELECT * FROM products WHERE 1=1");
        let mut count_query = String::from("SELECT COUNT(*) as count FROM products WHERE 1=1");

        // Prepare search pattern outside of query building
        let search_pattern = filters.search.as_ref().map(|s| format!("%{}%", s));

        // Add filters
        if filters.category.is_some() {
            query.push_str(" AND category = ?");
            count_query.push_str(" AND category = ?");
        }
        if let Some(is_active) = filters.is_active {
            query.push_str(&format!(" AND is_active = {}", is_active));
            count_query.push_str(&format!(" AND is_active = {}", is_active));
        }
        if filters.min_price.is_some() {
            query.push_str(" AND price >= ?");
            count_query.push_str(" AND price >= ?");
        }
        if filters.max_price.is_some() {
            query.push_str(" AND price <= ?");
            count_query.push_str(" AND price <= ?");
        }
        if search_pattern.is_some() {
            query.push_str(" AND (name LIKE ? OR description LIKE ? OR sku LIKE ?)");
            count_query.push_str(" AND (name LIKE ? OR description LIKE ? OR sku LIKE ?)");
        }

        query.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

        // Execute count query
        let mut count_q = sqlx::query_scalar::<_, i64>(&count_query);
        if let Some(ref category) = filters.category {
            count_q = count_q.bind(category);
        }
        if let Some(min_price) = filters.min_price {
            count_q = count_q.bind(min_price);
        }
        if let Some(max_price) = filters.max_price {
            count_q = count_q.bind(max_price);
        }
        if let Some(ref pattern) = search_pattern {
            count_q = count_q.bind(pattern).bind(pattern).bind(pattern);
        }

        let total = count_q.fetch_one(&self.pool).await? as u64;

        // Execute main query
        let mut main_q = sqlx::query_as::<_, Product>(&query);
        if let Some(ref category) = filters.category {
            main_q = main_q.bind(category);
        }
        if let Some(min_price) = filters.min_price {
            main_q = main_q.bind(min_price);
        }
        if let Some(max_price) = filters.max_price {
            main_q = main_q.bind(max_price);
        }
        if let Some(ref pattern) = search_pattern {
            main_q = main_q.bind(pattern).bind(pattern).bind(pattern);
        }
        main_q = main_q.bind(per_page as i64).bind(offset as i64);

        let products = main_q.fetch_all(&self.pool).await?;

        let total_pages = (total + per_page - 1) / per_page;

        Ok(PaginatedResponse {
            data: products.into_iter().map(ProductResponse::from).collect(),
            total,
            page,
            per_page,
            total_pages,
        })
    }

    async fn update(&self, id: &str, input: UpdateProductInput) -> Result<Product, AppError> {
        // First check if product exists
        let _ = self.find_by_id(id).await?;

        let mut updates = Vec::new();
        let mut query = String::from("UPDATE products SET ");

        if input.sku.is_some() {
            updates.push("sku = ?");
        }
        if input.name.is_some() {
            updates.push("name = ?");
        }
        if input.description.is_some() {
            updates.push("description = ?");
        }
        if input.price.is_some() {
            updates.push("price = ?");
        }
        if input.stock_quantity.is_some() {
            updates.push("stock_quantity = ?");
        }
        if input.category.is_some() {
            updates.push("category = ?");
        }
        if input.image_url.is_some() {
            updates.push("image_url = ?");
        }
        if input.is_active.is_some() {
            updates.push("is_active = ?");
        }

        if updates.is_empty() {
            return self.find_by_id(id).await;
        }

        query.push_str(&updates.join(", "));
        query.push_str(" WHERE id = ?");

        let mut q = sqlx::query(&query);

        if let Some(sku) = input.sku {
            q = q.bind(sku);
        }
        if let Some(name) = input.name {
            q = q.bind(name);
        }
        if let Some(description) = input.description {
            q = q.bind(description);
        }
        if let Some(price) = input.price {
            q = q.bind(price);
        }
        if let Some(stock_quantity) = input.stock_quantity {
            q = q.bind(stock_quantity);
        }
        if let Some(category) = input.category {
            q = q.bind(category);
        }
        if let Some(image_url) = input.image_url {
            q = q.bind(image_url);
        }
        if let Some(is_active) = input.is_active {
            q = q.bind(is_active);
        }

        q = q.bind(id);

        q.execute(&self.pool).await?;

        self.find_by_id(id).await
    }

    async fn delete(&self, id: &str) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM products WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Product with id '{}' not found",
                id
            )));
        }

        Ok(())
    }

    async fn update_stock(&self, id: &str, quantity_delta: i32) -> Result<Product, AppError> {
        sqlx::query(
            "UPDATE products SET stock_quantity = stock_quantity + ? WHERE id = ?"
        )
        .bind(quantity_delta)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.find_by_id(id).await
    }
}
