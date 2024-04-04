use greeter::greeter_server::{Greeter, GreeterServer};
use greeter::{HelloRequest, HelloResponse};
use tonic;
use tonic::transport::Server;

pub mod greeter {
    tonic::include_proto!("greeter");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("greeter_descriptor");
}

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: tonic::Request<HelloRequest>,
    ) -> Result<tonic::Response<HelloResponse>, tonic::Status> {
        println!("Received request from {:?}", request);

        let response = greeter::HelloResponse {
            message: format!("Hello {}!", request.into_inner().name).into(),
        };

        Ok(tonic::Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = MyGreeter::default();

    let service = tonic_reflection::server::Builder::configure() // Change this line
        .register_encoded_file_descriptor_set(greeter::FILE_DESCRIPTOR_SET)
        .build()?;

    println!("Starting gRPC server...");
    Server::builder()
        .add_service(service)
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
