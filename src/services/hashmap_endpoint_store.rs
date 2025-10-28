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
        // filter for active servers
        let active_endpoints: Vec<_> = self
            .endpoints
            .values()
            .filter(|ep| ep.active_server.load(Ordering::Relaxed))
            .collect();

        if active_endpoints.is_empty() {
            return Err(EndpointStoreError::NoEndpoints);
        }

        // let selected_endpoint = self.lowest_connection_index_selection();
        let selected_endpoint = self.round_robin_index_selection(active_endpoints);

        print!("Selected endpoint index: {}\n", selected_endpoint.uri);
        Ok(selected_endpoint.clone())
    }
}

impl HashmapEndpointStore {
    fn lowest_connection_index_selection(&self) -> &Endpoint {
        // find the enumerated endpoint with the minimum concurrent connections

        let x = self
            .endpoints
            .values()
            .enumerate()
            .min_by_key(|(_, ep)| ep.count_concurrent_connections.load(Ordering::Relaxed))
            .map(|(_, j)| j)
            .unwrap();
        x
    }

    fn round_robin_index_selection(&self, active_endpoints: Vec<&Endpoint>) -> Endpoint {
        if self.current_index.load(Ordering::Relaxed) >= active_endpoints.len() {
            self.current_index.store(0, Ordering::Relaxed);
        }

        let current_idx = self.current_index.fetch_add(1, Ordering::Relaxed);
        active_endpoints[current_idx % active_endpoints.len()].clone()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{atomic::AtomicBool, Arc};

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

        // Test that we get different endpoints on successive calls - round robin 1, 0, 1
        let first_endpoint = endpoint_store.get_next_endpoint().await.unwrap();
        let second_endpoint = endpoint_store.get_next_endpoint().await.unwrap();
        let third_endpoint = endpoint_store.get_next_endpoint().await.unwrap();

        print!(
            "First: {:?}, Second: {:?}, Third: {:?}",
            first_endpoint, second_endpoint, third_endpoint
        );

        assert_eq!(
            first_endpoint.uri,
            Uri::from_static("http://example-two.com")
        );
        assert_eq!(second_endpoint.uri, Uri::from_static("http://example.com"));
        assert_eq!(
            third_endpoint.uri,
            Uri::from_static("http://example-two.com")
        );
    }

    #[tokio::test]
    async fn test_get_endpoint_failed_server() {
        let mut endpoint_store = HashmapEndpointStore::default();
        let endpoint1 = Endpoint {
            uri: Uri::from_static("http://example.com"),
            count_success: Default::default(),
            count_failure: Default::default(),
            count_concurrent_connections: Default::default(),
            active_server: Arc::new(AtomicBool::new(false)), // inactive server
        };

        let endpoint2 = Endpoint {
            uri: Uri::from_static("http://example-two.com"),
            count_success: Default::default(),
            count_failure: Default::default(),
            count_concurrent_connections: Default::default(),
            active_server: Arc::new(AtomicBool::new(false)), // inactive server
        };

        // Add endpoint
        let _ = endpoint_store.add_endpoint(endpoint1).await;
        let _ = endpoint_store.add_endpoint(endpoint2).await;

        // Test that we get error when all servers are inactive
        let result = endpoint_store.get_next_endpoint().await;
        assert!(result.is_err());
    }
}
