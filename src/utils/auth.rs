// use axum_extra::extract::cookie::{Cookie, SameSite};
// use chrono::Utc;
// use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Validation};
// use serde::{Deserialize, Serialize};

// use crate::domain::email::Email;

// use super::constants::{JWT_COOKIE_NAME, JWT_SECRET};

// pub fn generate_auth_cookie(email: &Email) -> Result<Cookie<'static>, GenerateTokenError> {
//     let token = generate_auth_token(email)?;
//     Ok(create_auth_cookie(token))
// }

// fn create_auth_cookie(token: String) -> Cookie<'static> {
//     let cookie = Cookie::build((JWT_COOKIE_NAME, token))
//         .path("/")
//         .http_only(true)
//         .same_site(SameSite::Lax)
//         .build();

//     cookie
// }

// #[derive(Debug)]
// pub enum GenerateTokenError {
//     TokenError(jsonwebtoken::errors::Error),
//     UnexpectedError,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Claims {
//     pub sub: String,
//     pub exp: usize,
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::sync::Arc;
//     use tokio::sync::RwLock;

//     #[tokio::test]
//     async fn test_generate_auth_cookie() {
//         let email = Email::parse("test@example.com".to_owned()).unwrap();
//         let cookie = generate_auth_cookie(&email).unwrap();
//         assert_eq!(cookie.name(), JWT_COOKIE_NAME);
//         assert_eq!(cookie.value().split('.').count(), 3);
//         assert_eq!(cookie.path(), Some("/"));
//         assert_eq!(cookie.http_only(), Some(true));
//         assert_eq!(cookie.same_site(), Some(SameSite::Lax));
//     }

//     #[tokio::test]
//     async fn test_create_auth_cookie() {
//         let token = "test_token".to_owned();
//         let cookie = create_auth_cookie(token.clone());
//         assert_eq!(cookie.name(), JWT_COOKIE_NAME);
//         assert_eq!(cookie.value(), token);
//         assert_eq!(cookie.path(), Some("/"));
//         assert_eq!(cookie.http_only(), Some(true));
//         assert_eq!(cookie.same_site(), Some(SameSite::Lax));
//     }
// }
