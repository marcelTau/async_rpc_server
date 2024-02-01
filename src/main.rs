use keyvalue::{
    key_value_server::{KeyValue, KeyValueServer},
    RetrieveRequest, RetrieveResponse, StoreRequest, StoreResponse,
};
use sqlx::{migrate::MigrateDatabase, Row, Sqlite, SqlitePool};
use tokio::{sync::Semaphore, time::sleep};
use tonic::{transport::Server, Request, Response, Status};

pub mod keyvalue {
    tonic::include_proto!("keyvalue");
}

pub struct KeyValueService {
    conn: sqlx::Pool<Sqlite>,
}

const DB_URL: &str = "sqlite://kv.db";

/// rate limiting: 10 concurrent requests
static PERMITS: Semaphore = Semaphore::const_new(10);

impl KeyValueService {
    pub async fn new(db_url: &str) -> Self {
        if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
            println!("Creating database {}", db_url);
            match Sqlite::create_database(db_url).await {
                Ok(_) => println!("Create db success"),
                Err(error) => panic!("error: {}", error),
            }
        } else {
            println!("Database already exists");
        }

        let conn = SqlitePool::connect(db_url).await.unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS key_value_store (
            key TEXT UNIQUE,
            value TEXT
         )",
        )
        .execute(&conn)
        .await
        .expect("Failed to create table");

        Self { conn }
    }
}

#[tonic::async_trait]
impl KeyValue for KeyValueService {
    async fn store(
        &self,
        request: Request<StoreRequest>,
    ) -> Result<Response<StoreResponse>, Status> {
        let _permit = PERMITS.acquire().await.unwrap();

        // simulating some heavy work, for demonstration of rate limiting
        let _t = sleep(std::time::Duration::from_millis(200)).await;

        match sqlx::query("INSERT INTO key_value_store (key, value) VALUES ($1, $2)")
            .bind(&request.get_ref().key)
            .bind(&request.get_ref().value)
            .execute(&self.conn)
            .await
        {
            Ok(_) => Ok(Response::new(StoreResponse {})),
            Err(e) => Err(Status::already_exists(e.to_string())),
        }
    }

    async fn retrieve(
        &self,
        request: Request<RetrieveRequest>,
    ) -> Result<Response<RetrieveResponse>, Status> {
        let _permit = PERMITS.acquire().await.unwrap();

        // simulating some heavy work, for demonstration of rate limiting
        let _t = sleep(std::time::Duration::from_millis(200)).await;

        let rows = match sqlx::query("SELECT value FROM key_value_store WHERE key = $1")
            .bind(&request.get_ref().key)
            .fetch_one(&self.conn)
            .await
        {
            Ok(r) => r,
            Err(e) => return Err(Status::not_found(e.to_string())),
        };

        if rows.len() == 0 {
            return Err(Status::not_found("Key not found"));
        }

        let value: String = rows.get::<_, usize>(0);

        Ok(Response::new(RetrieveResponse { value }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kv_service = KeyValueService::new(DB_URL).await;
    let address = "[::1]:50051".parse().unwrap();

    Server::builder()
        .add_service(KeyValueServer::new(kv_service))
        .serve(address)
        .await?;
    Ok(())
}
