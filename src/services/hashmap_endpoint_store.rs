use crate::domain::{Email, Endpoint, EndpointStore, Password, UserStoreError};
use axum::http::Uri;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
pub struct HashmapEndpointStore {
    endpoints: HashMap<Email, Endpoint>,
    current_index: AtomicUsize,
}

#[async_trait::async_trait]
impl EndpointStore for HashmapEndpointStore {
    async fn add_endpoint(&mut self, user: Endpoint) -> Result<(), UserStoreError> {
        if self.endpoints.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        }
        self.endpoints.insert(user.email.clone(), user);
        Ok(())
    }

    async fn get_next_endpoint(&self) -> Result<Endpoint, UserStoreError> {
        let endpoints: Vec<_> = self.endpoints.values().collect();

        if endpoints.is_empty() {
            return Err(UserStoreError::UserNotFound);
        }

        let current_idx = self.current_index.fetch_add(1, Ordering::Relaxed);
        let index = current_idx % endpoints.len();

        Ok(endpoints[index].clone())
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        match self.endpoints.get(email) {
            Some(user) => {
                if user.password.eq(password) {
                    Ok(())
                } else {
                    Err(UserStoreError::InvalidCredentials)
                }
            }
            None => Err(UserStoreError::UserNotFound),
        }
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
            email: Email::parse("test@example.com".to_owned()).unwrap(),
            password: Password::parse("password".to_owned()).unwrap(),
            requires_2fa: false,
        };

        let endpoint2 = Endpoint {
            uri: Uri::from_static("http://example-two.com"),
            email: Email::parse("test@example.com".to_owned()).unwrap(),
            password: Password::parse("password".to_owned()).unwrap(),
            requires_2fa: false,
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
            email: Email::parse("test1@example.com".to_owned()).unwrap(),
            password: Password::parse("password1".to_owned()).unwrap(),
            requires_2fa: false,
        };

        let endpoint2 = Endpoint {
            uri: Uri::from_static("http://example2.com"),
            email: Email::parse("test2@example.com".to_owned()).unwrap(),
            password: Password::parse("password2".to_owned()).unwrap(),
            requires_2fa: false,
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
        // Should cycle back to first endpoint
        assert_eq!(first_endpoint.email, third_endpoint.email);
        assert_ne!(first_endpoint.email, second_endpoint.email);
    }

    // #[tokio::test]
    // async fn test_validate_user() {
    //     let mut user_store = HashmapEndpointStore::default();
    //     let email = Email::parse("test@example.com".to_owned()).unwrap();
    //     let password = Password::parse("password".to_owned()).unwrap();

    //     let user = Endpoint {
    //         email: email.clone(),
    //         password: password.clone(),
    //         requires_2fa: false,
    //     };

    //     // Test validating a user that exists with correct password
    //     user_store.endpoints.insert(email.clone(), user.clone());
    //     let result = user_store.validate_user(&email, &password).await;
    //     assert_eq!(result, Ok(()));

    //     // Test validating a user that exists with incorrect password
    //     let wrong_password = Password::parse("wrongpassword".to_owned()).unwrap();
    //     let result = user_store.validate_user(&email, &wrong_password).await;
    //     assert_eq!(result, Err(UserStoreError::InvalidCredentials));

    //     // Test validating a user that doesn't exist
    //     let result = user_store
    //         .validate_user(
    //             &Email::parse("nonexistent@example.com".to_string()).unwrap(),
    //             &password,
    //         )
    //         .await;

    //     assert_eq!(result, Err(UserStoreError::UserNotFound));
    // }
}
