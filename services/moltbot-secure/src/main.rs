//! Moltbot Secure - Encrypted Wrapper Layer
//! 
//! Provides a secure, encrypted proxy layer around Moltbot to protect:
//! - User communications and data
//! - API keys and credentials
//! - System access permissions
//! - Conversation history
//!
//! Security Features:
//! - Post-quantum hybrid encryption (ML-KEM + HQC + AES-256-GCM)
//! - End-to-end encryption for all communications
//! - Secure key exchange protocol
//! - Authentication and access control
//! - Encrypted session management
//! - Secure credential storage

mod encryption;
mod key_manager;
mod proxy;
mod auth;
mod config;
mod session;

use axum::{
    extract::State,
    routing::{get, post},
    Router,
    Json,
};
use std::sync::Arc;
use tracing::{info, error};
use tower_http::cors::CorsLayer;

use crate::config::Config;
use crate::session::SessionManager;
use crate::auth::AuthManager;
use crate::key_manager::KeyManager;
use crate::proxy::ProxyHandler;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub session_manager: Arc<SessionManager>,
    pub auth_manager: Arc<AuthManager>,
    pub key_manager: Arc<KeyManager>,
    pub proxy_handler: Arc<ProxyHandler>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "moltbot_secure=info,tower_http=debug".into()),
        )
        .init();

    info!("Starting Moltbot Secure - Encrypted Wrapper Layer");

    // Load configuration
    let config = Arc::new(Config::load()?);
    info!("Configuration loaded from: {:?}", config.config_path);

    // Initialize managers
    let key_manager = Arc::new(KeyManager::new(&config)?);
    let auth_manager = Arc::new(AuthManager::new(&config));
    let session_manager = Arc::new(SessionManager::new(&config));
    let proxy_handler = Arc::new(ProxyHandler::new(&config, key_manager.clone())?);

    let app_state = AppState {
        config: config.clone(),
        session_manager,
        auth_manager,
        key_manager,
        proxy_handler,
    };

    // Build application
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/auth/register", post(register_handler))
        .route("/api/v1/auth/login", post(login_handler))
        .route("/api/v1/session/create", post(create_session_handler))
        .route("/api/v1/proxy/message", post(proxy_message_handler))
        .route("/api/v1/keys/exchange", post(key_exchange_handler))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr = format!("{}:{}", config.bind_address, config.port);
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "moltbot-secure",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn register_handler(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    match state.auth_manager.register(&payload).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Registration error: {}", e);
            Err(axum::response::Response::builder()
                .status(400)
                .body(axum::body::Body::from(format!("{{\"error\":\"{}\"}}", e)))
                .unwrap()
                .into())
        }
    }
}

async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    match state.auth_manager.login(&payload).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Login error: {}", e);
            Err(axum::response::Response::builder()
                .status(401)
                .body(axum::body::Body::from(format!("{{\"error\":\"{}\"}}", e)))
                .unwrap()
                .into())
        }
    }
}

async fn create_session_handler(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    match state.session_manager.create_session(&payload).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Session creation error: {}", e);
            Err(axum::response::Response::builder()
                .status(400)
                .body(axum::body::Body::from(format!("{{\"error\":\"{}\"}}", e)))
                .unwrap()
                .into())
        }
    }
}

async fn proxy_message_handler(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    match state.proxy_handler.handle_message(&payload).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Proxy error: {}", e);
            Err(axum::response::Response::builder()
                .status(500)
                .body(axum::body::Body::from(format!("{{\"error\":\"{}\"}}", e)))
                .unwrap()
                .into())
        }
    }
}

async fn key_exchange_handler(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    match state.key_manager.exchange_keys(&payload).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Key exchange error: {}", e);
            Err(axum::response::Response::builder()
                .status(400)
                .body(axum::body::Body::from(format!("{{\"error\":\"{}\"}}", e)))
                .unwrap()
                .into())
        }
    }
}
