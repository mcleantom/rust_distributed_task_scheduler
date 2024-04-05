use coordinator::coordinator_service_server::{CoordinatorService, CoordinatorServiceServer};
use coordinator::{
    ClientTaskRequest, ClientTaskResponse, HeartbeatRequest, HeartbeatResponse,
    RegisterWorkerRequest, RegisterWorkerResponse, UpdateTaskStatusRequest,
    UpdateTaskStatusResponse,
};
use log::info;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tonic::transport::Channel;
use tonic::transport::Server;
use tonic::{self, Response};

pub mod coordinator {
    tonic::include_proto!("coordinator");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("coordinator_descriptor");
}

pub struct WorkerInfo {
    heartbeat_misses: u32,
    address: String,
    channel: Channel,
    active: bool,
}

pub struct MyCoordinator {
    pub db_connection_string: String,
    db_pool: Arc<sqlx::PgPool>,
    worker_pool: Arc<Mutex<Vec<WorkerInfo>>>,
    heartbeat_interval: u32,
    heartbeat_check_running: bool,
    max_heartbeat_misses: u32,
}

impl MyCoordinator {
    pub async fn new(db_connection_string: String) -> Self {
        let db_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_connection_string)
            .await
            .expect("Failed to connect to the database");

        Self {
            db_connection_string,
            db_pool: Arc::new(db_pool),
            worker_pool: Arc::new(Mutex::new(Vec::new())),
            heartbeat_interval: 10,
            heartbeat_check_running: false,
            max_heartbeat_misses: 5,
        }
    }

    async fn run_heartbeat_check(&mut self) {
        info!("Running heartbeat check loop");
        self.heartbeat_check_running = true;
        while (self.heartbeat_check_running) {
            sleep(Duration::from_secs(u64::from(self.heartbeat_interval))).await;
            let mut worker_pool = self.worker_pool.lock().await;
            for worker_info in &mut *worker_pool {
                worker_info.heartbeat_misses += 1;
                if worker_info.active & (worker_info.heartbeat_misses >= self.max_heartbeat_misses)
                {
                    worker_info.active = false;
                    info!("Worker at address {} is inactive", worker_info.address);
                }
            }
        }
    }
}

#[tonic::async_trait]
impl CoordinatorService for MyCoordinator {
    async fn register_worker(
        &self,
        request: tonic::Request<RegisterWorkerRequest>,
    ) -> Result<tonic::Response<RegisterWorkerResponse>, tonic::Status> {
        let mut worker_pool = self.worker_pool.lock().await;

        let reply = RegisterWorkerResponse {
            worker_id: worker_pool.len() as u32,
            heartbeat_interval: self.heartbeat_interval,
        };

        let channel = Channel::from_shared(request.get_ref().address.clone())
            .expect("Invalid worker address")
            .connect()
            .await
            .expect("Failed to connect to worker");

        worker_pool.push(WorkerInfo {
            heartbeat_misses: 0,
            address: request.get_ref().address.clone(),
            channel,
            active: true,
        });

        Ok(Response::new(reply))
    }

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
        let input = request.get_ref();
        let worker_id = input.worker_id;

        info!("Got heartbeat from worker {}", worker_id);

        if (worker_id as usize) < self.worker_pool.lock().await.len() {
            let mut worker_pool = self.worker_pool.lock().await;
            worker_pool[worker_id as usize].heartbeat_misses = 0;
        } else {
            return Err(tonic::Status::invalid_argument("Invalid worker ID"));
        }

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
    env_logger::init();
    let addr = "[::1]:50051".parse()?;
    let greeter =
        MyCoordinator::new("postgres://postgres:password@localhost:5432/postgres".to_string())
            .await;

    // let heartbeat_task = tokio::spawn(async move {
    //     greeter.run_heartbeat_check().await;
    // })

    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(coordinator::FILE_DESCRIPTOR_SET)
        .build()?;

    info!("Starting gRPC server...");
    let server_task = Server::builder()
        .add_service(service)
        .add_service(CoordinatorServiceServer::new(greeter))
        .serve(addr);

    tokio::select! {
        _ = server_task => {
            info!("Server terminated");
        }
    }

    Ok(())
}
