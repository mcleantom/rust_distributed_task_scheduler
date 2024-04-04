use coordinator::coordinator_service_server::{CoordinatorService, CoordinatorServiceServer};
use coordinator::{
    ClientTaskRequest, ClientTaskResponse, HeartbeatRequest, HeartbeatResponse,
    UpdateTaskStatusRequest, UpdateTaskStatusResponse,
};
use tonic::transport::Server;
use tonic::{self, Response};

pub mod coordinator {
    tonic::include_proto!("coordinator");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("coordinator_descriptor");
}

#[derive(Debug, Default)]
pub struct MyCoordinator {}

#[tonic::async_trait]
impl CoordinatorService for MyCoordinator {
    async fn submit_task(
        &self,
        request: tonic::Request<ClientTaskRequest>,
    ) -> Result<tonic::Response<ClientTaskResponse>, tonic::Status> {
        let reply = ClientTaskResponse {
            message: "hi".to_string(),
            task_id: "hello".to_string(),
        };
        Ok(Response::new(reply))
    }

    async fn send_heartbeat(
        &self,
        request: tonic::Request<HeartbeatRequest>,
    ) -> Result<tonic::Response<HeartbeatResponse>, tonic::Status> {
        let reply = HeartbeatResponse { acknowledged: true };
        Ok(Response::new(reply))
    }

    async fn update_task_status(
        &self,
        request: tonic::Request<UpdateTaskStatusRequest>,
    ) -> Result<tonic::Response<UpdateTaskStatusResponse>, tonic::Status> {
        let reply = UpdateTaskStatusResponse { success: true };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = MyCoordinator::default();

    let service = tonic_reflection::server::Builder::configure() // Change this line
        .register_encoded_file_descriptor_set(coordinator::FILE_DESCRIPTOR_SET)
        .build()?;

    println!("Starting gRPC server...");
    Server::builder()
        .add_service(service)
        .add_service(CoordinatorServiceServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
