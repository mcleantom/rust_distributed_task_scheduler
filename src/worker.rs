use coordinator::worker_service_server::{WorkerService, WorkerServiceServer};
use coordinator::coordinator_service_client::CoordinatorServiceClient;
use coordinator::{
    RegisterWorkerRequest, TaskRequest, TaskResponse
};
use tonic::transport::Server;
use tonic::transport::Channel;
use tonic::{self, Response, Request};


pub mod coordinator {
    tonic::include_proto!("coordinator");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("coordinator_descriptor");
}

#[derive(Clone)]
pub struct MyWorker {
    address: String,
    coordinator_address: String,
    client: Option<CoordinatorServiceClient<Channel>>,
}

impl MyWorker {
    pub async fn new(address: String, coordinator_address: String) -> Self {
        Self {
            address: address,
            coordinator_address: coordinator_address.clone(),
            client: None,
        }
    }

    async fn register_worker(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut new_client = CoordinatorServiceClient::connect(self.coordinator_address.clone()).await?;

        let http_address = format!("http://{}", self.address);
        
        let request = tonic::Request::new(RegisterWorkerRequest {
            address: http_address.clone(),
        });

        println!("Registering worker with address: {}", http_address);
        new_client.register_worker(request).await?;
        println!("Worker registered with address: {}", http_address);

        self.client = Some(new_client);
        Ok(())
    }
}

#[tonic::async_trait]
impl WorkerService for MyWorker {
    async fn submit_task(
        &self,
        request: Request<TaskRequest>,
    ) -> Result<Response<TaskResponse>, tonic::Status> {
        let reply = TaskResponse {
            task_id: "123".into(),
            message: "Task received".into(),
            success: true
        };
        Ok(Response::new(reply))
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: String = "[::1]:50053".parse()?;
    let coordinator_address: String = "http://[::1]:50051".parse()?;
    let worker = MyWorker::new(address.clone(), coordinator_address.clone()).await;

    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(coordinator::FILE_DESCRIPTOR_SET)
        .build()?;

    println!("Starting gRPC worker");
    let addr = "[::1]:50053".parse()?;

    let mut worker = MyWorker::new(address.clone(), coordinator_address.clone()).await;

    let server = Server::builder()
        .add_service(service)
        .add_service(WorkerServiceServer::new(worker.clone()))
        .serve(addr);

    // register worker in spawned task
    tokio::spawn(async move {
        worker.register_worker().await.unwrap();
    });

    server.await?;

    Ok(())

}