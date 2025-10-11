use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
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
    // let password = match Password::parse(request.password) {
    //     Ok(password) => password,
    //     Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    // };

    // let email = match Email::parse(request.email) {
    //     Ok(email) => email,
    //     Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    // };

    // if user_store.validate_user(&email, &password).await.is_err() {
    //     return (jar, Err(AuthAPIError::IncorrectCredentials));
    // }

    print!("Routeme called with request: {}\n", request.uri());

    let endpoint_store = &state.endpoint_store.read().await;

    let end_point = match endpoint_store.get_next_endpoint().await {
        Ok(end_point) => end_point,
        Err(_) => return Err(RouterError::IncorrectCredentials),
    };

    // Make HTTP request to the endpoint's URI
    let client = reqwest::Client::new();

    let response = match client
        .get(end_point.uri.to_string())
        // .json(&request) // Forward the login request
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return Err(RouterError::UnexpectedError),
    };

    return Ok((StatusCode::OK, Json(response.text().await.unwrap())));
}

async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), RouterError>,
) {
    // let auth_cookie = match generate_auth_cookie(email) {
    //     Ok(cookie) => cookie,
    //     Err(_) => return (jar, Err(AuthAPIError::UnexpectedError)),
    // };

    // let updated_jar = jar.add(auth_cookie);

    (jar, Ok((StatusCode::OK, Json(LoginResponse::RegularAuth))))
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
