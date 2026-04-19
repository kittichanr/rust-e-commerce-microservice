use common_libs::proto::auth::greeter_server::Greeter;
use common_libs::proto::auth::{HelloReply, HelloRequest};
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    type SayHelloStreamStream = ReceiverStream<Result<HelloReply, Status>>;

    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request: {:?}", request);
        let name = request.into_inner().name;
        let reply = HelloReply {
            message: format!("Hello {}!", name),
        };

        Ok(Response::new(reply))
    }

    // server-streaming RPC
    async fn say_hello_stream(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<Self::SayHelloStreamStream>, Status> {
        let name = request.into_inner().name;

        let (tx, rx) = mpsc::channel(4);

        tokio::spawn(async move {
            let greetings = vec![
                format!("Hello, {}! (1/3)", name),
                format!("Hi again, {}! (2/3)", name),
                format!("Greetings, {}! (3/3)", name),
            ];

            for greeting in greetings {
                if tx.send(Ok(HelloReply { message: greeting })).await.is_err() {
                    break;
                }
                sleep(Duration::from_secs(1)).await;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
