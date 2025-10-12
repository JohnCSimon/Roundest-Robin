use axum::{
    body::Body,
    extract::{Request, State},
    http::{StatusCode, Uri},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{Email, Password, RouterError},
};

pub async fn routeme(
    State(state): State<AppState>,
    jar: CookieJar,
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

    let combined_uri: Uri = combined_uri_string
        .parse()
        .map_err(|_| RouterError::UnexpectedError)?;

    let response = match client
        .get(combined_uri_string)
        // .json(&request) // Forward the login request
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return Err(RouterError::UnexpectedError),
    };

    Ok((StatusCode::OK, Json(response.text().await.unwrap())))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
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
