use std::sync::Arc;
use tokio::sync::RwLock;

use roundest_robin_router::{
    app_state::AppState,
    domain::{Endpoint, EndpointStore},
    services::hashmap_endpoint_store::HashmapEndpointStore,
    utils::constants::prod,
    Application,
};

#[tokio::main]
async fn main() {
    let endpoint_store = Arc::new(RwLock::new(HashmapEndpointStore::default()));

    for port in 7001..=7005 {
        // PURELY FOR TESTING PURPOSES - MAKE THIS REAL
        let uri = format!("http://localhost:{}", port).parse().unwrap();
        let endpoint = Endpoint::new(uri);

        endpoint_store
            .write()
            .await
            .add_endpoint(endpoint)
            .await
            .unwrap();
    }

    // // // bad endpoint for testing failed server scenario
    // // let bad_uri = "http://localhost:7005".parse().unwrap();
    // // let bad_endpoint = Endpoint::new(bad_uri);
    // // bad_endpoint.deactivate();

    // endpoint_store
    //     .write()
    //     .await
    //     .add_endpoint(bad_endpoint)
    //     .await
    //     .unwrap();

    let app_state = AppState::new(endpoint_store);

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
