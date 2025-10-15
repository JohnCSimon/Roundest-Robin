use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{app_state::AppState, domain::RouterError};

// TODO: this thing doesn't properly pass along the request body etc.
pub async fn routeme(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<impl IntoResponse, RouterError> {
    print!("Routeme called with request: {}\n", request.uri());

    let endpoint_store = &state.endpoint_store.read().await;

    let end_point = match endpoint_store.get_next_endpoint().await {
        Ok(end_point) => end_point,
        Err(_) => return Err(RouterError::IncorrectCredentials),
    };
    println!(
        "Forwarding request to endpoint: {} {}",
        end_point.uri,
        request.uri()
    );

    // Make HTTP request to the endpoint's URI
    let client = reqwest::Client::new();

    let combined_uri_string = format!(
        "{}{}",
        end_point.uri,
        request
            .uri()
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("")
    );

    let response = match client
        .get(combined_uri_string)
        // .json(&request) // Forward the login request
        .send()
        .await
    {
        Ok(response) => {
            end_point.incr_success();
            response
        }
        Err(_) => {
            end_point.incr_failure();
            return Err(RouterError::UnexpectedError);
        }
    };

    Ok((StatusCode::OK, Json(response.text().await.unwrap())))
}

pub async fn print_stats(State(state): State<AppState>) -> Result<impl IntoResponse, RouterError> {
    let endpoint_store = &state.endpoint_store.read().await;

    let endpoints = endpoint_store.get_all_endpoints().await.unwrap();

    let stats: Vec<EndpointStats> = endpoints
        .into_iter()
        .map(|ep| EndpointStats {
            uri: ep.uri.to_string(),
            count_success: ep
                .count_success
                .load(std::sync::atomic::Ordering::Relaxed)
                .to_string(),
            count_failure: ep
                .count_failure
                .load(std::sync::atomic::Ordering::Relaxed)
                .to_string(),
        })
        .collect();

    Ok((StatusCode::OK, Json(stats)))
}

#[derive(Debug, Serialize)]
pub struct EndpointStats {
    pub uri: String,
    pub count_success: String,
    pub count_failure: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}
