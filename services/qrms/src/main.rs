//! QRMS - Quantum Resistance Model System
//! Low-fidelity prototype demonstrating:
//! - QRM: Quantum Resistance Monitor
//! - APQC: Adaptive PQC Layer
//! - TEE Sequencer
//! - Chain State

mod qrm;
mod apqc;
mod crypto;
mod sequencer;
mod chain;
mod state;
mod handlers;

use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::state::AppState;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "qrms=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting QRMS - Quantum Resistance Model System");

    // Create shared state
    let state = Arc::new(AppState::new());

    // Start background simulation task
    let sim_state = state.clone();
    tokio::spawn(async move {
        state::run_simulation(sim_state).await;
    });

    // Build router
    let app = Router::new()
        // API routes
        .route("/api/status", get(handlers::get_status))
        .route("/api/qrm/history", get(handlers::get_qrm_history))
        .route("/api/blocks", get(handlers::get_blocks))
        .route("/api/inject_threat", post(handlers::inject_threat))
        .route("/api/simulation/start", post(handlers::start_simulation))
        .route("/api/simulation/stop", post(handlers::stop_simulation))
        .route("/api/inject_high_threat", post(handlers::inject_high_threat))
        // WebSocket for real-time updates
        .route("/ws", get(handlers::websocket_handler))
        // Serve static files
        .nest_service("/", ServeDir::new("static"))
        // CORS
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any))
        // State
        .with_state(state);

    let addr = "0.0.0.0:5050";
    tracing::info!("Server running at http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
