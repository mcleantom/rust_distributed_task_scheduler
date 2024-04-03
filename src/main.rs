mod scheduler;

#[tokio::main]
async fn main() {
    let server = scheduler::SchedulerServer::new(
        "postgres://postgres:password@localhost:5432/postgres".to_string(),
    )
    .await;

    if let Err(err) = server.start_server().await {
        eprintln!("Failed to start server: {}", err);
    }
}
