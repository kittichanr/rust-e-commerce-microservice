use crate::{error::GatewayError, middleware::Claims, AppState};
use actix_web::{web, HttpResponse};
use common_libs::proto::product::{
    product_client::ProductClient, CreateProductRequest, GetProductRequest, ListProductsRequest,
    UpdateProductRequest,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateProductInput {
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub price: i64,
    pub stock_quantity: i32,
    pub category: Option<String>,
    pub image_url: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductInput {
    pub sku: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<i64>,
    pub stock_quantity: Option<i32>,
    pub category: Option<String>,
    pub image_url: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub product_id: String,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub price: i64,
    pub stock_quantity: i32,
    pub category: Option<String>,
    pub image_url: Option<String>,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct ProductListQuery {
    pub category: Option<String>,
    pub is_active: Option<bool>,
    pub min_price: Option<i64>,
    pub max_price: Option<i64>,
    pub search: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ProductListResponse {
    pub products: Vec<ProductResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

pub async fn list_products(
    state: web::Data<AppState>,
    query: web::Query<ProductListQuery>,
) -> Result<HttpResponse, GatewayError> {
    let mut client = ProductClient::connect(state.config.services.product_service_url.clone())
        .await
        .map_err(|e| GatewayError::ServiceUnavailable(format!("Product service: {}", e)))?;

    let request = tonic::Request::new(ListProductsRequest {
        category: query.category.clone(),
        is_active: query.is_active,
        min_price: query.min_price,
        max_price: query.max_price,
        search: query.search.clone(),
        page: query.page,
        per_page: query.per_page,
    });
    let response = client.list_products(request).await?.into_inner();

    let products: Vec<ProductResponse> = response
        .products
        .into_iter()
        .map(|p| ProductResponse {
            product_id: p.product_id,
            sku: p.sku,
            name: p.name,
            description: p.description,
            price: p.price,
            stock_quantity: p.stock_quantity,
            category: p.category,
            image_url: p.image_url,
            is_active: p.is_active,
            created_at: p.created_at,
            updated_at: p.updated_at,
        })
        .collect();

    Ok(HttpResponse::Ok().json(ProductListResponse {
        products,
        total: response.total,
        page: response.page,
        per_page: response.per_page,
        total_pages: response.total_pages,
    }))
}

pub async fn get_product(
    state: web::Data<AppState>,
    product_id: web::Path<String>,
) -> Result<HttpResponse, GatewayError> {
    let mut client = ProductClient::connect(state.config.services.product_service_url.clone())
        .await
        .map_err(|e| GatewayError::ServiceUnavailable(format!("Product service: {}", e)))?;

    let request = tonic::Request::new(GetProductRequest {
        product_id: product_id.into_inner(),
    });

    let response = client.get_product(request).await?.into_inner();
    let product = response.product.ok_or(GatewayError::NotFound)?;

    Ok(HttpResponse::Ok().json(ProductResponse {
        product_id: product.product_id,
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
    }))
}

pub async fn create_product(
    state: web::Data<AppState>,
    input: web::Json<CreateProductInput>,
    _claims: Claims,
) -> Result<HttpResponse, GatewayError> {
    let mut client = ProductClient::connect(state.config.services.product_service_url.clone())
        .await
        .map_err(|e| GatewayError::ServiceUnavailable(format!("Product service: {}", e)))?;

    let request = tonic::Request::new(CreateProductRequest {
        sku: input.sku.clone(),
        name: input.name.clone(),
        description: input.description.clone(),
        price: input.price,
        stock_quantity: input.stock_quantity,
        category: input.category.clone(),
        image_url: input.image_url.clone(),
        is_active: input.is_active,
    });

    let response = client.create_product(request).await?.into_inner();
    let product = response
        .product
        .ok_or(GatewayError::Internal("Product not returned".to_string()))?;

    Ok(HttpResponse::Created().json(ProductResponse {
        product_id: product.product_id,
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
    }))
}

pub async fn update_product(
    state: web::Data<AppState>,
    product_id: web::Path<String>,
    input: web::Json<UpdateProductInput>,
    _claims: Claims,
) -> Result<HttpResponse, GatewayError> {
    let mut client = ProductClient::connect(state.config.services.product_service_url.clone())
        .await
        .map_err(|e| GatewayError::ServiceUnavailable(format!("Product service: {}", e)))?;

    let request = tonic::Request::new(UpdateProductRequest {
        product_id: product_id.into_inner(),
        sku: input.sku.clone(),
        name: input.name.clone(),
        description: input.description.clone(),
        price: input.price,
        stock_quantity: input.stock_quantity,
        category: input.category.clone(),
        image_url: input.image_url.clone(),
        is_active: input.is_active,
    });

    let response = client.update_product(request).await?.into_inner();
    let product = response.product.ok_or(GatewayError::NotFound)?;

    Ok(HttpResponse::Ok().json(ProductResponse {
        product_id: product.product_id,
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
    }))
}

// Configure public product routes (no authentication required)
pub fn configure_public(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(list_products))
        .route("/{id}", web::get().to(get_product));
}

// Configure protected product routes (authentication required)
pub fn configure_protected(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::post().to(create_product))
        .route("/{id}", web::put().to(update_product));
}
