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
    domain::{errors::AppError, models::CreateProductInput},
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

    // Stub implementations for other methods (not implemented yet)
    async fn get_product(
        &self,
        _request: Request<GetProductRequest>,
    ) -> Result<Response<GetProductResponse>, Status> {
        Err(Status::unimplemented("get_product not implemented yet"))
    }

    async fn update_product(
        &self,
        _request: Request<UpdateProductRequest>,
    ) -> Result<Response<UpdateProductResponse>, Status> {
        Err(Status::unimplemented("update_product not implemented yet"))
    }

    async fn delete_product(
        &self,
        _request: Request<DeleteProductRequest>,
    ) -> Result<Response<DeleteProductResponse>, Status> {
        Err(Status::unimplemented("delete_product not implemented yet"))
    }

    async fn list_products(
        &self,
        _request: Request<ListProductsRequest>,
    ) -> Result<Response<ListProductsResponse>, Status> {
        Err(Status::unimplemented("list_products not implemented yet"))
    }
}
