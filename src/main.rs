use keyvalue::{
    key_value_server::{KeyValue, KeyValueServer},
    RetrieveRequest, RetrieveResponse, StoreRequest, StoreResponse,
};
use tokio::{sync::Semaphore, time::sleep};
use tonic::{transport::Server, Request, Response, Status};

mod db_handler;
use db_handler::{DBHandler, DBError};

pub mod keyvalue {
    tonic::include_proto!("keyvalue");
}

pub struct KeyValueService {
    db: DBHandler,
}

/// rate limiting: 10 concurrent requests
static PERMITS: Semaphore = Semaphore::const_new(10);

impl KeyValueService {
    pub async fn new() -> Result<Self, DBError> {
        let db = DBHandler::connect().await?;
        Ok(Self { db })
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
        self.db.store(&request.get_ref().key, &request.get_ref().value).await
    }

    async fn retrieve(
        &self,
        request: Request<RetrieveRequest>,
    ) -> Result<Response<RetrieveResponse>, Status> {
        let _permit = PERMITS.acquire().await.unwrap();

        // simulating some heavy work, for demonstration of rate limiting
        let _t = sleep(std::time::Duration::from_millis(200)).await;
        self.db.retrieve(&request.get_ref().key).await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kv_service = KeyValueService::new().await?;
    let address = "[::1]:50051".parse()?;

    Server::builder()
        .add_service(KeyValueServer::new(kv_service))
        .serve(address)
        .await?;
    Ok(())
}
