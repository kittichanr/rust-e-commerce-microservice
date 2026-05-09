use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

// ========== Product Models ==========

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: String,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub price: i64, // Price in cents
    pub stock_quantity: i32,
    pub category: Option<String>,
    pub image_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateProductInput {
    #[validate(length(min = 1, max = 100))]
    pub sku: String,

    #[validate(length(min = 1, max = 255))]
    pub name: String,

    pub description: Option<String>,

    #[validate(range(min = 0))]
    pub price: i64,

    #[validate(range(min = 0))]
    pub stock_quantity: i32,

    #[validate(length(max = 100))]
    pub category: Option<String>,

    #[validate(length(max = 500))]
    pub image_url: Option<String>,

    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateProductInput {
    #[validate(length(min = 1, max = 100))]
    pub sku: Option<String>,

    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,

    pub description: Option<String>,

    #[validate(range(min = 0))]
    pub price: Option<i64>,

    #[validate(range(min = 0))]
    pub stock_quantity: Option<i32>,

    #[validate(length(max = 100))]
    pub category: Option<String>,

    #[validate(length(max = 500))]
    pub image_url: Option<String>,

    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductResponse {
    pub id: String,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub price: i64,
    pub stock_quantity: i32,
    pub category: Option<String>,
    pub image_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Product> for ProductResponse {
    fn from(product: Product) -> Self {
        Self {
            id: product.id,
            sku: product.sku,
            name: product.name,
            description: product.description,
            price: product.price,
            stock_quantity: product.stock_quantity,
            category: product.category,
            image_url: product.image_url,
            is_active: product.is_active,
            created_at: product.created_at,
            updated_at: product.updated_at,
        }
    }
}

// ========== Pagination & Filters ==========

#[derive(Debug, Clone, Deserialize)]
pub struct ProductFilters {
    pub category: Option<String>,
    pub is_active: Option<bool>,
    pub min_price: Option<i64>,
    pub max_price: Option<i64>,
    pub search: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

impl Default for ProductFilters {
    fn default() -> Self {
        Self {
            category: None,
            is_active: Some(true),
            min_price: None,
            max_price: None,
            search: None,
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
