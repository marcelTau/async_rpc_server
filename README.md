# Async RPC Server

## Requirements
- **Build an Async RPC Server**: Using the tokio crate, build an asynchronous RPC server.
- **Database Integration**: Integrate the server with a database to handle arbitrary data storage and retrieval.
- **Key-Value Store Functionalities**:
  - Allow users to store arbitrary strings with a unique key.
  - Enable fetching of these strings using their respective keys.
- **Rate Limiting**: Implement rate limiting on the server to manage the load and ensure efficient resource utilization.

## Technologies used
**Tokio**: Async Runtime  
**Tonic**: RPC server  
**Sqlx**: Framework for DB connection  
**Sqlite**: Database  
**grpcurl**: Testing script  
