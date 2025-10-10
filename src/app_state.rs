use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::EndpointStore;

pub type EndpointStoreType = Arc<RwLock<dyn EndpointStore + Send + Sync>>;

#[derive(Clone)]
pub struct AppState {
    pub endpoint_store: EndpointStoreType,
}

impl AppState {
    pub fn new(endpoint_store: EndpointStoreType) -> Self {
        Self { endpoint_store }
    }
}
