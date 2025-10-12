use std::sync::Arc;
use tokio::sync::RwLock;

use auth_service::{
    app_state::AppState,
    domain::{Endpoint, EndpointStore},
    services::hashmap_endpoint_store::HashmapEndpointStore,
    utils::constants::prod,
    Application,
};

#[tokio::main]
async fn main() {
    let endpoint_store = Arc::new(RwLock::new(HashmapEndpointStore::default()));

    let endpoint = Endpoint {
        uri: "http://localhost:9000".parse().unwrap(),
    };
    endpoint_store
        .write()
        .await
        .add_endpoint(endpoint)
        .await
        .unwrap();
    let endpoint = Endpoint {
        uri: "http://localhost:9001".parse().unwrap(),
    };
    endpoint_store
        .write()
        .await
        .add_endpoint(endpoint)
        .await
        .unwrap();
    let app_state = AppState::new(endpoint_store);

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
