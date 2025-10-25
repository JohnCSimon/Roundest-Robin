use crate::domain::{Endpoint, EndpointStore, EndpointStoreError};
use axum::http::Uri;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
pub struct HashmapEndpointStore {
    endpoints: HashMap<Uri, Endpoint>,
    current_index: AtomicUsize,
}

#[async_trait::async_trait]
impl EndpointStore for HashmapEndpointStore {
    async fn add_endpoint(&mut self, endpoint: Endpoint) -> Result<(), EndpointStoreError> {
        if self.endpoints.contains_key(&endpoint.uri) {
            return Err(EndpointStoreError::EndpointAlreadyExists);
        }
        self.endpoints.insert(endpoint.uri.clone(), endpoint);
        Ok(())
    }

    async fn get_all_endpoints(&self) -> Result<Vec<Endpoint>, EndpointStoreError> {
        Ok(self.endpoints.values().cloned().collect())
    }

    async fn get_next_endpoint(&self) -> Result<Endpoint, EndpointStoreError> {
        let endpoints: Vec<_> = self.endpoints.values().collect();

        if endpoints.is_empty() {
            return Err(EndpointStoreError::NoEndpoints);
        }

        let index = self.lowest_connection_index_selection();
        // let index = self.round_robin_index_selection(endpoints.len());

        print!("Selected endpoint index: {}\n", index);
        Ok(endpoints[index].clone())
    }
}

impl HashmapEndpointStore {
    fn lowest_connection_index_selection(&self) -> usize {
        // find the enumerated endpoint with the minimum concurrent connections

        self.endpoints
            .values()
            .enumerate()
            .filter(|x| x.1.active_server.load(Ordering::Relaxed)) // only consider active servers
            .min_by_key(|(_, ep)| ep.count_concurrent_connections.load(Ordering::Relaxed))
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    fn round_robin_index_selection(&self, len: usize) -> usize {
        if self.current_index.load(Ordering::Relaxed) >= len {
            self.current_index.store(0, Ordering::Relaxed);
        }

        let current_idx = self.current_index.fetch_add(1, Ordering::Relaxed);
        current_idx % len
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicBool;

    use super::*;

    #[tokio::test]
    async fn test_add_endpoint() {
        let mut endpoint_store = HashmapEndpointStore::default();

        let endpoint1 = Endpoint::new(Uri::from_static("http://example.com"));
        let endpoint2 = Endpoint::new(Uri::from_static("http://example-two.com"));

        // Test adding a new user
        let result = endpoint_store.add_endpoint(endpoint1).await;
        assert!(result.is_ok());

        // Test adding an existing user
        let result = endpoint_store.add_endpoint(endpoint2).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_endpoint() {
        let mut endpoint_store = HashmapEndpointStore::default();
        let endpoint1 = Endpoint::new(Uri::from_static("http://example.com"));
        let endpoint2 = Endpoint::new(Uri::from_static("http://example-two.com"));

        // Test getting endpoint from empty store
        let result = endpoint_store.get_next_endpoint().await;
        assert!(result.is_err());

        // Add endpoints
        let _ = endpoint_store.add_endpoint(endpoint1).await;
        let _ = endpoint_store.add_endpoint(endpoint2).await;

        // Test that we get different endpoints on successive calls
        let first_endpoint = endpoint_store.get_next_endpoint().await.unwrap();
        let second_endpoint = endpoint_store.get_next_endpoint().await.unwrap();
        let third_endpoint = endpoint_store.get_next_endpoint().await.unwrap();

        print!(
            "First: {:?}, Second: {:?}, Third: {:?}",
            first_endpoint, second_endpoint, third_endpoint
        );
    }
}
