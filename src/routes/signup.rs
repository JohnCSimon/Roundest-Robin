use axum::{
    extract::State,
    http::{uri, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{Email, Endpoint, Password, RouterError},
};

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, RouterError> {
    let uri = uri::Uri::from_static("http://example.com");
    let user = Endpoint::new(uri);

    let mut user_store = state.endpoint_store.write().await;

    if user_store.get_next_endpoint().await.is_ok() {
        return Err(RouterError::UserAlreadyExists);
    }

    if user_store.add_endpoint(user).await.is_err() {
        return Err(RouterError::UnexpectedError);
    }

    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SignupResponse {
    pub message: String,
}
