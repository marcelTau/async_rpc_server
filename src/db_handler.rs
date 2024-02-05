use sqlx::{migrate::MigrateDatabase, Row, Sqlite, SqlitePool};
use crate::keyvalue::{RetrieveResponse, StoreResponse};
use tonic::{Response, Status};

#[derive(Debug)]
pub enum DBError {
    CreateDatabaseFailed(String),
    CreateTableFailed(String),
}

impl std::error::Error for DBError {}

impl std::fmt::Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct DBHandler {
    conn: sqlx::Pool<Sqlite>,
}

impl DBHandler {
    const DEFAULT_DB_URL: &'static str  = "sqlite://kv.db";

    pub async fn connect() -> Result<Self, DBError> {
        let db_url = std::env::var("DB_URL").unwrap_or(Self::DEFAULT_DB_URL.to_string());

        if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
            println!("Creating database {}", db_url);
            match Sqlite::create_database(&db_url).await {
                Ok(_) => println!("Create db success"),
                Err(e) => return Err(DBError::CreateDatabaseFailed(e.to_string())),
            }
        } else {
            println!("Database already exists");
        }

        let conn = SqlitePool::connect(&db_url).await.unwrap();

        match sqlx::query(
            "CREATE TABLE IF NOT EXISTS key_value_store (
            key TEXT UNIQUE,
            value TEXT
         )",
        )
        .execute(&conn)
        .await
        {
            Ok(_) => (),
            Err(e) => return Err(DBError::CreateTableFailed(e.to_string())),
        }

        Ok(Self { conn })
    }

    pub async fn store(&self, key: &str, value: &str) -> Result<Response<StoreResponse>, Status> {
        match sqlx::query("INSERT INTO key_value_store (key, value) VALUES ($1, $2)")
            .bind(key)
            .bind(value)
            .execute(&self.conn)
            .await
        {
            Ok(_) => Ok(Response::new(StoreResponse {})),
            Err(e) => Err(Status::already_exists(e.to_string())),
        }
    }

    pub async fn retrieve(&self, key: &str) -> Result<Response<RetrieveResponse>, Status> {
        let rows = match sqlx::query("SELECT value FROM key_value_store WHERE key = $1")
            .bind(key)
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
