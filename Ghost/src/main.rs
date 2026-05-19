mod handlers;
mod state;
mod types;

use axum::{
    extract::State,
    routing::get,
    Router,
};
use handlers::handle_socket_upgrade;
use state::create_app_state;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Initialize tracing for diagnostics
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Create the global application state
    let app_state = create_app_state();

    // Build the router with the WebSocket upgrade route
    let app = Router::new()
        .route("/ws", get(handle_socket_upgrade))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Bind to 0.0.0.0:8080
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to 8080");

    tracing::info!("Ghost server listening on {}", addr);

    // Run the server
    axum::serve(
        listener,
        app,
    )
    .await
    .expect("Server error");
}
