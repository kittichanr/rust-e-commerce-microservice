use std::sync::Arc;

use common_libs::proto::order::order_server::Order as OrderService;
use common_libs::proto::order::{
    CancelOrderRequest, CancelOrderResponse, CreateOrderRequest, CreateOrderResponse,
    GetOrderRequest, GetOrderResponse, ListOrdersRequest, ListOrdersResponse,
    UpdateOrderStatusRequest, UpdateOrderStatusResponse,
};
use tonic::{Request, Response, Status};

use crate::repository::order::OrderRepository;

pub struct MyOrderService {
    order_repo: Arc<dyn OrderRepository>,
}

impl MyOrderService {
    pub fn new(order_repo: Arc<dyn OrderRepository>) -> Self {
        Self { order_repo }
    }
}

#[tonic::async_trait]
impl OrderService for MyOrderService {
    async fn create_order(
        &self,
        _request: Request<CreateOrderRequest>,
    ) -> Result<Response<CreateOrderResponse>, Status> {
        todo!("Implement create_order")
    }

    async fn get_order(
        &self,
        _request: Request<GetOrderRequest>,
    ) -> Result<Response<GetOrderResponse>, Status> {
        todo!("Implement get_order")
    }

    async fn update_order_status(
        &self,
        _request: Request<UpdateOrderStatusRequest>,
    ) -> Result<Response<UpdateOrderStatusResponse>, Status> {
        todo!("Implement update_order_status")
    }

    async fn cancel_order(
        &self,
        _request: Request<CancelOrderRequest>,
    ) -> Result<Response<CancelOrderResponse>, Status> {
        todo!("Implement cancel_order")
    }

    async fn list_orders(
        &self,
        _request: Request<ListOrdersRequest>,
    ) -> Result<Response<ListOrdersResponse>, Status> {
        todo!("Implement list_orders")
    }
}
