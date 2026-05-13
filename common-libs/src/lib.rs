pub mod proto {

    pub mod auth {
        tonic::include_proto!("auth");
    }

    pub mod product {
        tonic::include_proto!("product");
    }

    pub mod order {
        tonic::include_proto!("order");
    }
}

pub mod events;
