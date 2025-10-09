use axum::http::Uri;

use super::{Email, Password};

#[derive(Clone, Debug, PartialEq)]
pub struct Endpoint {
    pub uri: Uri,
    pub email: Email,
    pub password: Password,
    pub requires_2fa: bool,
}

impl Endpoint {
    pub fn new(uri: Uri, email: Email, password: Password, requires_2fa: bool) -> Self {
        Self {
            uri,
            email,
            password,
            requires_2fa,
        }
    }
}
