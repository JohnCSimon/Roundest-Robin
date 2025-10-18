use rand::Rng;

use super::Endpoint;

#[async_trait::async_trait]
pub trait EndpointStore {
    async fn add_endpoint(&mut self, endpoint: Endpoint) -> Result<(), EndpointStoreError>;
    async fn get_next_endpoint(&self) -> Result<Endpoint, EndpointStoreError>;
    async fn get_all_endpoints(&self) -> Result<Vec<Endpoint>, EndpointStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum EndpointStoreError {
    EndpointAlreadyExists,
    NoEndpoints,
    InvalidCredentials,
    UnexpectedError,
}
