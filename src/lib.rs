use std::error::Error;

use app_state::AppState;
use axum::{
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    serve::Serve,
    Json, Router,
};
use domain::RouterError;
use routes::routeme;
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

use crate::routes::print_stats;

pub mod app_state;
pub mod domain;
pub mod routes;
pub mod services;
pub mod utils;

pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let allowed_origins = [
            "http://localhost:8000".parse()?,
            "http://[YOUR_DROPLET_IP]:8000".parse()?,
        ];

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_credentials(true)
            .allow_origin(allowed_origins);

        let router = Router::new()
            .route("/printstats", get(print_stats))
            .fallback(routeme)
            .with_state(app_state)
            .layer(cors);

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for RouterError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            RouterError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            RouterError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            RouterError::IncorrectCredentials => {
                (StatusCode::UNAUTHORIZED, "Incorrect credentials")
            }
            RouterError::MissingToken => (StatusCode::BAD_REQUEST, "Missing auth token"),
            RouterError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid auth token"),
            RouterError::UnexpectedError => (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error"),
        };
        let body = Json(ErrorResponse {
            error: error_message.to_string(),
        });
        (status, body).into_response()
    }
}
