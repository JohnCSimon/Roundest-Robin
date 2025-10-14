use axum::http::Uri;

#[derive(Clone, Debug, PartialEq)]
pub struct Endpoint {
    pub uri: Uri,
    pub count_success: usize,
    pub count_failure: usize,
}

impl Endpoint {
    pub fn new(uri: Uri) -> Self {
        Self {
            uri,
            count_success: 0,
            count_failure: 0,
        }
    }
}
