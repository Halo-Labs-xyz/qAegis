//! QRMS - Quantum Resistance Model System
//! Low-fidelity prototype demonstrating:
//! - QVM: Quantum Virtual Machine (Oracle Layer)
//! - QRM: Quantum Resistance Monitor
//! - APQC: Adaptive PQC Layer
//! - TEE Sequencer (Aegis-TEE with Phala redundancy)
//! - Chain State
//!
//! Architecture:
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    QUANTUM AEGIS PROTOCOL STACK                 │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                 │
//! │  ┌─────────────────────────────────────────────────────────┐    │
//! │  │               QVM ORACLE LAYER (qvm.rs)                 │    │
//! │  │  ┌─────────────────┐ ┌─────────────────┐ ┌───────────┐  │    │
//! │  │  │ Quantum Circuit │ │ Noise Simulator │ │   Risk    │  │    │
//! │  │  │   Simulator     │ │ (Willow/Weber)  │ │  Oracle   │  │    │
//! │  │  └────────┬────────┘ └────────┬────────┘ └─────┬─────┘  │    │
//! │  │           └───────────────────┼────────────────┘        │    │
//! │  │                               ▼                         │    │
//! │  │  ┌─────────────────────────────────────────────────┐    │    │
//! │  │  │           QRMS (qrm.rs, apqc.rs)                │    │    │
//! │  │  │  12 Threat Categories • Hybrid PQC Signatures   │    │    │
//! │  │  └────────────────────────┬────────────────────────┘    │    │
//! │  └───────────────────────────┼─────────────────────────────┘    │
//! │                              ▼                                  │
//! │  ┌─────────────────────────────────────────────────────────┐    │
//! │  │           AEGIS-TEE LAYER (aegis_tee.rs)                │    │
//! │  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌─────────┐  │    │
//! │  │  │ Encrypted │ │  Asset    │ │ Migration │ │ Intelli │  │    │
//! │  │  │  Mempool  │ │Protection │ │  System   │ │ Order   │  │    │
//! │  │  └───────────┘ └───────────┘ └───────────┘ └─────────┘  │    │
//! │  │                              │                            │    │
//! │  │  ┌───────────────────────────▼────────────────────────┐  │    │
//! │  │  │      Phala Network Redundancy (Optional)          │  │    │
//! │  │  └───────────────────────────────────────────────────┘  │    │
//! │  └─────────────────────────────────────────────────────────┘    │
//! │                              ▼                                  │
//! │  ┌─────────────────────────────────────────────────────────┐    │
//! │  │              BLOCKCHAIN LAYER (chain.rs)                │    │
//! │  │  ┌───────────────┐ ┌───────────────┐ ┌──────────────┐   │    │
//! │  │  │ L2 Sequencer  │ │    Smart      │ │   Rollup     │   │    │
//! │  │  │    Batches    │ │   Contracts   │ │  Settlement  │   │    │
//! │  │  └───────────────┘ └───────────────┘ └──────────────┘   │    │
//! │  └─────────────────────────────────────────────────────────┘    │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

mod aegis_tee;
mod apqc;
mod chain;
mod crypto;
mod governance_plane;
mod handlers;
mod hybrid_engine;
mod hybrid_quantum;
mod lean_sig;
mod lean_vm;
mod phala_deploy;
#[allow(deprecated)]
mod phala_tee; // Deprecated: kept for backward compatibility, use aegis_tee instead
mod qrm;
mod qvm;
mod security_plane;
mod sequencer;
mod state;

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
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

    // Start background purge task for transient hybrid execution artifacts
    let purge_state = state.clone();
    tokio::spawn(async move {
        state::run_hybrid_purge(purge_state).await;
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
        .route(
            "/api/inject_high_threat",
            post(handlers::inject_high_threat),
        )
        // LeanSig API routes
        .route("/api/lean/sign", post(handlers::lean_sig_sign))
        .route("/api/lean/verify", post(handlers::lean_sig_verify))
        .route(
            "/api/lean/generate-keys",
            post(handlers::lean_sig_generate_keys),
        )
        // LeanVM API routes
        .route("/api/lean-vm/execute", post(handlers::lean_vm_execute))
        .route("/api/lean-vm/poseidon", post(handlers::lean_vm_poseidon))
        .route("/api/lean-vm/status", get(handlers::lean_vm_status))
        // Hybrid quantum-classical ROMA-inspired API
        .route("/api/hybrid/solve", post(handlers::hybrid_solve))
        .route(
            "/api/hybrid/executions/:execution_id",
            get(handlers::hybrid_get_execution),
        )
        .route(
            "/api/hybrid/transparency",
            get(handlers::hybrid_transparency_log),
        )
        // WebSocket for real-time updates
        .route("/ws", get(handlers::websocket_handler))
        // Serve static files
        .nest_service("/", ServeDir::new("static"))
        // CORS
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
        // State
        .with_state(state);

    let addr = "0.0.0.0:5050";
    tracing::info!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
