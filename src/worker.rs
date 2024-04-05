use coordinator::coordinator_service_client::CoordinatorServiceClient;
use coordinator::worker_service_server::{WorkerService, WorkerServiceServer};
use coordinator::{
    HeartbeatRequest, HeartbeatResponse, RegisterWorkerRequest, TaskRequest, TaskResponse,
};
use log::{error, info};
use tokio::time::{sleep, Duration};
use tonic::transport::Channel;
use tonic::transport::Server;
use tonic::{self, Request, Response};

pub mod coordinator {
    tonic::include_proto!("coordinator");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("coordinator_descriptor");
}

#[derive(Clone)]
pub struct MyWorker {
    address: String,
    http_address: String,
    worker_id: u32,
    coordinator_address: String,
    client: Option<CoordinatorServiceClient<Channel>>,
    heartbeat_interval: u64,
    heartbeat_running: bool,
}

impl MyWorker {
    pub async fn new(address: String, coordinator_address: String) -> Self {
        Self {
            address: address.clone(),
            http_address: format!("http://{}", address),
            worker_id: 0,
            coordinator_address: coordinator_address,
            client: None,
            heartbeat_interval: 0,
            heartbeat_running: false,
        }
    }

    async fn register_worker(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut new_client =
            CoordinatorServiceClient::connect(self.coordinator_address.clone()).await?;

        let request = tonic::Request::new(RegisterWorkerRequest {
            address: self.http_address.clone(),
        });

        info!("Registering worker with address: {}", self.http_address);
        let response = new_client.register_worker(request).await?;
        info!("Worker registered with address: {}", self.http_address);

        let output = response.get_ref();
        self.heartbeat_interval = u64::from(output.heartbeat_interval);
        self.worker_id = output.worker_id;

        self.client = Some(new_client);

        Ok(())
    }

    async fn run_heartbeat(&mut self) {
        info!("Starting heartbeat");
        self.heartbeat_running = true;
        while self.heartbeat_running {
            self.send_heartbeat().await;
            sleep(Duration::from_secs(self.heartbeat_interval)).await;
        }
    }

    async fn send_heartbeat(&mut self) {
        info!("Sending heartbeat");

        let request = tonic::Request::new(HeartbeatRequest {
            worker_id: self.worker_id,
            address: self.http_address.clone(),
        });

        let response = self.client.as_mut().unwrap().send_heartbeat(request).await;

        match (response) {
            Ok(_) => {
                info!("Heartbeat sent successfully");
            }
            Err(err) => {
                error!("Failed to send heartbeat: {:?}", err);
            }
        }
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
            success: true,
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let address: String = "[::1]:50053".parse()?;
    let coordinator_address: String = "http://[::1]:50051".parse()?;
    let worker = MyWorker::new(address.clone(), coordinator_address.clone()).await;

    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(coordinator::FILE_DESCRIPTOR_SET)
        .build()?;

    info!("Starting gRPC worker");
    let addr = "[::1]:50053".parse()?;

    let mut worker = MyWorker::new(address.clone(), coordinator_address.clone()).await;

    let server = Server::builder()
        .add_service(service)
        .add_service(WorkerServiceServer::new(worker.clone()))
        .serve(addr);

    // register worker in spawned task
    tokio::spawn(async move {
        worker.register_worker().await.unwrap();
        worker.run_heartbeat().await;
    });

    server.await?;

    Ok(())
}
