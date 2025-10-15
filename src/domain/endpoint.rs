use std::{
    sync::atomic::{AtomicUsize, Ordering},
    sync::Arc,
};

use axum::http::Uri;

#[derive(Clone, Debug)]
pub struct Endpoint {
    pub uri: Uri,
    pub count_success: Arc<AtomicUsize>,
    pub count_failure: Arc<AtomicUsize>,
}

impl Endpoint {
    pub fn new(uri: Uri) -> Self {
        Self {
            uri,
            count_success: Arc::new(AtomicUsize::new(0)),
            count_failure: Arc::new(AtomicUsize::new(0)),
        }
    }
    pub fn incr_success(&self) {
        self.count_success.fetch_add(1, Ordering::Relaxed);
    }

    pub fn incr_failure(&self) {
        self.count_failure.fetch_add(1, Ordering::Relaxed);
    }

    pub fn success_count(&self) -> usize {
        self.count_success.load(Ordering::Relaxed)
    }

    pub fn failure_count(&self) -> usize {
        self.count_failure.load(Ordering::Relaxed)
    }
}
