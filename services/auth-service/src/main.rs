use auth_service::server::MyAuth;
use common_libs::proto::auth::auth_server::AuthServer;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let my_auth = MyAuth::default();

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(AuthServer::new(my_auth))
        .serve(addr)
        .await?;

    Ok(())
}
