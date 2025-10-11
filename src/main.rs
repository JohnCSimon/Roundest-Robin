use std::sync::Arc;
use tokio::sync::RwLock;

use auth_service::{
    app_state::AppState, services::hashmap_endpoint_store::HashmapEndpointStore,
    utils::constants::prod, Application,
};

#[tokio::main]
async fn main() {
    let endpoint_store = Arc::new(RwLock::new(HashmapEndpointStore::default()));

    let app_state = AppState::new(endpoint_store);

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
