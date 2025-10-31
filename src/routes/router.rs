use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{app_state::AppState, domain::RouterError};

pub async fn routeme(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<impl IntoResponse, RouterError> {
    let endpoint_store = &state.endpoint_store.read().await;

    // check for dead servers before selecting next endpoint
    endpoint_store.check_for_dead_servers().await;
    let end_point = match endpoint_store.get_next_endpoint().await {
        Ok(end_point) => {
            end_point.increase_concurrent_connection_count();
            end_point
        }
        Err(_) => {
            return Err(RouterError::IncorrectCredentials);
        }
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
            end_point.decrease_concurrent_connection_count();
            end_point.incr_success();
            response
        }
        Err(_) => {
            end_point.decrease_concurrent_connection_count();
            end_point.incr_failure();
            return Err(RouterError::UnexpectedError);
        }
    };

    let response_text = response.text().await.unwrap();

    // TODO: this thing doesn't properly pass along the request body etc.
    // this needs to pass the content-type and other headers too into what is returned form this function

    let converted_response = Response::builder()
        .header("content-type", "text/html; charset=utf-8")
        .body(Body::from(response_text))
        .unwrap();

    Ok((StatusCode::OK, converted_response))
}

pub async fn print_stats(State(state): State<AppState>) -> Result<impl IntoResponse, RouterError> {
    let endpoint_store = &state.endpoint_store.read().await;

    let endpoints = endpoint_store.get_all_endpoints().await.unwrap();
    let container_stats = crate::domain::get_docker_stats().await.unwrap();
    let stats: Vec<EndpointStats> = endpoints
        .into_iter()
        .map(|ep| EndpointStats {
            uri: ep.uri.to_string(),
            count_success: ep.count_success.load(std::sync::atomic::Ordering::Relaxed),
            count_failure: ep.count_failure.load(std::sync::atomic::Ordering::Relaxed),
            count_concurrent_connections: ep
                .count_concurrent_connections
                .load(std::sync::atomic::Ordering::Relaxed),
            active_server: ep.active_server.load(std::sync::atomic::Ordering::Relaxed),
            cpu_percentage: container_stats
                .get(&ep.uri.to_string())
                .map_or(0.0, |stats| stats.cpu_percentage),
            memory_usage: container_stats
                .get(&ep.uri.to_string())
                .map_or(0, |stats| stats.memory_usage.try_into().unwrap()),
            memory_limit: container_stats
                .get(&ep.uri.to_string())
                .map_or(0, |stats| stats.memory_limit.try_into().unwrap()),
            memory_percentage: container_stats
                .get(&ep.uri.to_string())
                .map_or(0.0, |stats| stats.memory_percentage),
            network_rx_bytes: container_stats
                .get(&ep.uri.to_string())
                .map_or(0, |stats| stats.network_rx_bytes.try_into().unwrap()),
            network_tx_bytes: container_stats
                .get(&ep.uri.to_string())
                .map_or(0, |stats| stats.network_tx_bytes.try_into().unwrap()),
        })
        .collect();

    Ok((StatusCode::OK, Json(stats)))
}

#[derive(Debug, Serialize)]
pub struct EndpointStats {
    pub uri: String,
    pub count_success: usize,
    pub count_failure: usize,
    pub count_concurrent_connections: usize,
    pub active_server: bool,
    pub cpu_percentage: f64,
    pub memory_usage: usize,
    pub memory_limit: usize,
    pub memory_percentage: f64,
    pub network_rx_bytes: usize,
    pub network_tx_bytes: usize,
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
