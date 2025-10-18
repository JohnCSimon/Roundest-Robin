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

        if self.current_index.load(Ordering::Relaxed) == endpoints.len() {
            self.current_index.store(0, Ordering::Relaxed);
        }
        let current_idx = self.current_index.fetch_add(1, Ordering::Relaxed);
        let index = current_idx % endpoints.len();

        print!("Selected endpoint index: {}\n", index);
        Ok(endpoints[index].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_endpoint() {
        let mut endpoint_store = HashmapEndpointStore::default();
        let endpoint1 = Endpoint {
            uri: Uri::from_static("http://example.com"),
            count_success: AtomicUsize::new(0).into(),
            count_failure: AtomicUsize::new(0).into(),
            count_concurrent_connections: AtomicUsize::new(0).into(),
        };

        let endpoint2 = Endpoint {
            uri: Uri::from_static("http://example-two.com"),
            count_success: AtomicUsize::new(0).into(),
            count_failure: AtomicUsize::new(0).into(),
            count_concurrent_connections: AtomicUsize::new(0).into(),
        };

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
        let endpoint1 = Endpoint {
            uri: Uri::from_static("http://example1.com"),
            count_success: AtomicUsize::new(0).into(),
            count_failure: AtomicUsize::new(0).into(),
            count_concurrent_connections: AtomicUsize::new(0).into(),
        };

        let endpoint2 = Endpoint {
            uri: Uri::from_static("http://example2.com"),
            count_success: AtomicUsize::new(0).into(),
            count_failure: AtomicUsize::new(0).into(),
            count_concurrent_connections: AtomicUsize::new(0).into(),
        };

        // Test getting endpoint from empty store
        let result = endpoint_store.get_next_endpoint().await;
        assert!(result.is_err());

        // Add endpoints
        let _ = endpoint_store.add_endpoint(endpoint1.clone()).await;
        let _ = endpoint_store.add_endpoint(endpoint2.clone()).await;

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
