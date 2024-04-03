use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use std::sync::Arc;
use std::time::SystemTime;

struct CommandRequest {
    command: String,
    scheduled_at: SystemTime,
}

struct Task {
    id: String,
    command: String,
    scheduled_at: SystemTime,
    picked_at: Option<SystemTime>,
    started_at: Option<SystemTime>,
    completed_at: Option<SystemTime>,
    failed_at: Option<SystemTime>,
}

#[get("/task")]
async fn task() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

pub struct SchedulerServer {
    pub db_connection_string: String,
    db_pool: Arc<sqlx::PgPool>,
    http_server: Option<actix_web::dev::Server>,
}

impl SchedulerServer {
    pub async fn new(db_connection_string: String) -> Self {
        let db_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_connection_string)
            .await
            .expect("Failed to connect to the database");
        Self {
            db_connection_string,
            db_pool: Arc::new(db_pool),
            http_server: None,
        }
    }

    pub async fn start_server(self) -> std::io::Result<()> {
        HttpServer::new(|| App::new().service(task))
            .bind(("127.0.0.1", 8080))?
            .run()
            .await
    }
}

#[tokio::test]
async fn test_scheduler_server_new_async() {
    let server =
        SchedulerServer::new("postgres://postgres:password@localhost:5432/postgres".to_string())
            .await;

    assert_eq!(
        server.db_connection_string,
        "postgres://postgres:password@localhost:5432/postgres"
    );
}
