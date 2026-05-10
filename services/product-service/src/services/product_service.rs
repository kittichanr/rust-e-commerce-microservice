use std::sync::Arc;

use common_libs::proto::product::product_server::Product as ProductService;
use common_libs::proto::product::{
    CreateProductRequest, CreateProductResponse, DeleteProductRequest, DeleteProductResponse,
    GetProductRequest, GetProductResponse, ListProductsRequest, ListProductsResponse, ProductInfo,
    UpdateProductRequest, UpdateProductResponse,
};
use tonic::{Request, Response, Status};
use validator::Validate;

use crate::{
    domain::{errors::AppError, models::CreateProductInput, models::UpdateProductInput},
    repository::product::ProductRepository,
};

pub struct MyProductService {
    product_repo: Arc<dyn ProductRepository>,
}

impl MyProductService {
    pub fn new(product_repo: Arc<dyn ProductRepository>) -> Self {
        Self { product_repo }
    }
}

#[tonic::async_trait]
impl ProductService for MyProductService {
    async fn create_product(
        &self,
        request: Request<CreateProductRequest>,
    ) -> Result<Response<CreateProductResponse>, Status> {
        let req = request.into_inner();

        // Convert proto request to domain input
        let input = CreateProductInput {
            sku: req.sku,
            name: req.name,
            description: req.description,
            price: req.price,
            stock_quantity: req.stock_quantity,
            category: req.category,
            image_url: req.image_url,
            is_active: req.is_active,
        };

        // Validate input
        if let Err(e) = input.validate() {
            return Ok(Response::new(CreateProductResponse {
                success: false,
                message: format!("Validation failed: {}", e),
                product: None,
            }));
        }

        // Create product
        match self.product_repo.create(input).await {
            Ok(product) => {
                let product_info = ProductInfo {
                    product_id: product.id,
                    sku: product.sku,
                    name: product.name,
                    description: product.description,
                    price: product.price,
                    stock_quantity: product.stock_quantity,
                    category: product.category,
                    image_url: product.image_url,
                    is_active: product.is_active,
                    created_at: product.created_at.timestamp(),
                    updated_at: product.updated_at.timestamp(),
                };

                Ok(Response::new(CreateProductResponse {
                    success: true,
                    message: "Product created successfully".to_string(),
                    product: Some(product_info),
                }))
            }
            Err(AppError::Conflict(msg)) => Ok(Response::new(CreateProductResponse {
                success: false,
                message: msg,
                product: None,
            })),
            Err(AppError::Validation(msg)) => Ok(Response::new(CreateProductResponse {
                success: false,
                message: msg,
                product: None,
            })),
            Err(e) => {
                tracing::error!("Failed to create product: {:?}", e);
                Ok(Response::new(CreateProductResponse {
                    success: false,
                    message: "Internal server error".to_string(),
                    product: None,
                }))
            }
        }
    }

    async fn get_product(
        &self,
        request: Request<GetProductRequest>,
    ) -> Result<Response<GetProductResponse>, Status> {
        let req = request.into_inner();

        // Validate product_id is provided
        if req.product_id.is_empty() {
            return Ok(Response::new(GetProductResponse {
                success: false,
                message: "Product ID is required".to_string(),
                product: None,
            }));
        }

        // Get product
        match self.product_repo.find_by_id(&req.product_id).await {
            Ok(product) => {
                let product_info = ProductInfo {
                    product_id: product.id,
                    sku: product.sku,
                    name: product.name,
                    description: product.description,
                    price: product.price,
                    stock_quantity: product.stock_quantity,
                    category: product.category,
                    image_url: product.image_url,
                    is_active: product.is_active,
                    created_at: product.created_at.timestamp(),
                    updated_at: product.updated_at.timestamp(),
                };

                Ok(Response::new(GetProductResponse {
                    success: true,
                    message: "Product retrieved successfully".to_string(),
                    product: Some(product_info),
                }))
            }
            Err(AppError::NotFound(msg)) => Ok(Response::new(GetProductResponse {
                success: false,
                message: msg,
                product: None,
            })),
            Err(e) => {
                tracing::error!("Failed to get product: {:?}", e);
                Ok(Response::new(GetProductResponse {
                    success: false,
                    message: "Internal server error".to_string(),
                    product: None,
                }))
            }
        }
    }

    async fn update_product(
        &self,
        request: Request<UpdateProductRequest>,
    ) -> Result<Response<UpdateProductResponse>, Status> {
        let req = request.into_inner();

        // Validate product_id is provided
        if req.product_id.is_empty() {
            return Ok(Response::new(UpdateProductResponse {
                success: false,
                message: "Product ID is required".to_string(),
                product: None,
            }));
        }

        // Convert proto request to domain input
        let input = UpdateProductInput {
            sku: req.sku,
            name: req.name,
            description: req.description,
            price: req.price,
            stock_quantity: req.stock_quantity,
            category: req.category,
            image_url: req.image_url,
            is_active: req.is_active,
        };

        // Validate input
        if let Err(e) = input.validate() {
            return Ok(Response::new(UpdateProductResponse {
                success: false,
                message: format!("Validation failed: {}", e),
                product: None,
            }));
        }

        // Update product
        match self.product_repo.update(&req.product_id, input).await {
            Ok(product) => {
                let product_info = ProductInfo {
                    product_id: product.id,
                    sku: product.sku,
                    name: product.name,
                    description: product.description,
                    price: product.price,
                    stock_quantity: product.stock_quantity,
                    category: product.category,
                    image_url: product.image_url,
                    is_active: product.is_active,
                    created_at: product.created_at.timestamp(),
                    updated_at: product.updated_at.timestamp(),
                };

                Ok(Response::new(UpdateProductResponse {
                    success: true,
                    message: "Product updated successfully".to_string(),
                    product: Some(product_info),
                }))
            }
            Err(AppError::NotFound(msg)) => Ok(Response::new(UpdateProductResponse {
                success: false,
                message: msg,
                product: None,
            })),
            Err(AppError::Conflict(msg)) => Ok(Response::new(UpdateProductResponse {
                success: false,
                message: msg,
                product: None,
            })),
            Err(AppError::Validation(msg)) => Ok(Response::new(UpdateProductResponse {
                success: false,
                message: msg,
                product: None,
            })),
            Err(e) => {
                tracing::error!("Failed to update product: {:?}", e);
                Ok(Response::new(UpdateProductResponse {
                    success: false,
                    message: "Internal server error".to_string(),
                    product: None,
                }))
            }
        }
    }

    async fn delete_product(
        &self,
        request: Request<DeleteProductRequest>,
    ) -> Result<Response<DeleteProductResponse>, Status> {
        let req = request.into_inner();

        // Validate product_id is provided
        if req.product_id.is_empty() {
            return Ok(Response::new(DeleteProductResponse {
                success: false,
                message: "Product ID is required".to_string(),
            }));
        }

        // Delete product
        match self.product_repo.delete(&req.product_id).await {
            Ok(_) => Ok(Response::new(DeleteProductResponse {
                success: true,
                message: "Product deleted successfully".to_string(),
            })),
            Err(AppError::NotFound(msg)) => Ok(Response::new(DeleteProductResponse {
                success: false,
                message: msg,
            })),
            Err(e) => {
                tracing::error!("Failed to delete product: {:?}", e);
                Ok(Response::new(DeleteProductResponse {
                    success: false,
                    message: "Internal server error".to_string(),
                }))
            }
        }
    }

    async fn list_products(
        &self,
        request: Request<ListProductsRequest>,
    ) -> Result<Response<ListProductsResponse>, Status> {
        let req = request.into_inner();

        // Map gRPC request to domain filters
        let filters = crate::domain::models::ProductFilters {
            category: req.category,
            is_active: req.is_active,
            min_price: req.min_price,
            max_price: req.max_price,
            search: req.search,
            page: req.page,
            per_page: req.per_page,
        };

        // Get products from repository
        match self.product_repo.find_all(filters).await {
            Ok(paginated) => {
                let products: Vec<ProductInfo> = paginated
                    .data
                    .into_iter()
                    .map(|product| ProductInfo {
                        product_id: product.id,
                        sku: product.sku,
                        name: product.name,
                        description: product.description,
                        price: product.price,
                        stock_quantity: product.stock_quantity,
                        category: product.category,
                        image_url: product.image_url,
                        is_active: product.is_active,
                        created_at: product.created_at.timestamp(),
                        updated_at: product.updated_at.timestamp(),
                    })
                    .collect();

                Ok(Response::new(ListProductsResponse {
                    success: true,
                    message: "Products retrieved successfully".to_string(),
                    products,
                    total: paginated.total,
                    page: paginated.page,
                    per_page: paginated.per_page,
                    total_pages: paginated.total_pages,
                }))
            }
            Err(e) => {
                tracing::error!("Failed to list products: {:?}", e);
                Ok(Response::new(ListProductsResponse {
                    success: false,
                    message: "Internal server error".to_string(),
                    products: vec![],
                    total: 0,
                    page: 0,
                    per_page: 0,
                    total_pages: 0,
                }))
            }
        }
    }
}
