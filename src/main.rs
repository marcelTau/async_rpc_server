use keyvalue::{
    key_value_server::{KeyValue, KeyValueServer},
    RetrieveRequest, RetrieveResponse, StoreRequest, StoreResponse,
};
use tokio::time::sleep;
use tonic::{transport::Server, Request, Response, Status};

mod db_handler;
use db_handler::{DBError, DBHandler};

use leaky_bucket::RateLimiter;

pub mod keyvalue {
    tonic::include_proto!("keyvalue");
}

pub struct KeyValueService {
    db: DBHandler,
    rate_limiter: RateLimiter,
}

impl KeyValueService {
    pub async fn new() -> Result<Self, DBError> {
        let db = DBHandler::connect().await?;
        let rate_limiter = RateLimiter::builder().max(100).initial(0).refill(2).build();
        Ok(Self { db, rate_limiter })
    }
}

#[tonic::async_trait]
impl KeyValue for KeyValueService {
    async fn store(
        &self,
        request: Request<StoreRequest>,
    ) -> Result<Response<StoreResponse>, Status> {
        self.rate_limiter.acquire_one().await;
        // simulating some heavy work, for demonstration of rate limiting
        let _t = sleep(std::time::Duration::from_millis(200)).await;
        self.db
            .store(&request.get_ref().key, &request.get_ref().value)
            .await
    }

    async fn retrieve(
        &self,
        request: Request<RetrieveRequest>,
    ) -> Result<Response<RetrieveResponse>, Status> {
        self.rate_limiter.acquire_one().await;
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
