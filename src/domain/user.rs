use axum::http::Uri;

#[derive(Clone, Debug, PartialEq)]
pub struct Endpoint {
    pub uri: Uri,
}

impl Endpoint {
    pub fn new(uri: Uri) -> Self {
        Self { uri }
    }
}
