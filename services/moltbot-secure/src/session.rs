//! Session Management
//!
//! Manages encrypted sessions between clients and the proxy

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

use crate::config::Config;
use crate::encryption::EncryptionKeyPair;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    #[serde(skip)]
    pub encryption_keypair: Option<EncryptionKeyPair>,
    pub last_activity: DateTime<Utc>,
    pub message_count: u64,
}

pub struct SessionManager {
    config: Arc<Config>,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    user_sessions: Arc<RwLock<HashMap<String, Vec<String>>>>, // user_id -> [session_ids]
}

impl SessionManager {
    pub fn new(config: &Arc<Config>) -> Self {
        Self {
            config: config.clone(),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            user_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new session
    pub async fn create_session(&self, payload: &serde_json::Value) -> Result<serde_json::Value> {
        let user_id = payload.get("user_id")
            .and_then(|v| v.as_str())
            .context("Missing user_id")?;
        let _token = payload.get("token")
            .and_then(|v| v.as_str())
            .context("Missing token")?;

        // Verify token (simplified - in production, use proper auth)
        // For now, we'll just check if it's a valid format

        // Check session limit
        let user_sessions = self.user_sessions.read().await;
        if let Some(session_ids) = user_sessions.get(user_id) {
            if session_ids.len() >= self.config.session.max_sessions_per_user as usize {
                return Err(anyhow::anyhow!("Maximum sessions per user reached"));
            }
        }
        drop(user_sessions);

        // Create session
        let session_id = uuid::Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::minutes(
            self.config.session.session_timeout_minutes as i64
        );

        let encryption_keypair = if self.config.session.enable_session_encryption {
            Some(EncryptionKeyPair::generate())
        } else {
            None
        };

        let session = Session {
            session_id: session_id.clone(),
            user_id: user_id.to_string(),
            created_at: Utc::now(),
            expires_at,
            encryption_keypair,
            last_activity: Utc::now(),
            message_count: 0,
        };

        // Store session
        self.sessions.write().await.insert(session_id.clone(), session.clone());

        // Associate with user
        self.user_sessions.write().await
            .entry(user_id.to_string())
            .or_insert_with(Vec::new)
            .push(session_id.clone());

        Ok(serde_json::json!({
            "success": true,
            "session_id": session_id,
            "expires_at": session.expires_at.timestamp(),
            "encryption_enabled": self.config.session.enable_session_encryption
        }))
    }

    /// Get session
    pub async fn get_session(&self, session_id: &str) -> Option<Session> {
        self.sessions.read().await
            .get(session_id)
            .cloned()
    }

    /// Update session activity
    pub async fn update_activity(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_activity = Utc::now();
            session.message_count += 1;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found"))
        }
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) {
        let now = Utc::now();
        let mut sessions = self.sessions.write().await;
        let mut user_sessions = self.user_sessions.write().await;

        let expired_ids: Vec<String> = sessions.iter()
            .filter(|(_, session)| session.expires_at < now)
            .map(|(id, _)| id.clone())
            .collect();

        for session_id in &expired_ids {
            if let Some(session) = sessions.remove(session_id) {
                if let Some(user_sess) = user_sessions.get_mut(&session.user_id) {
                    user_sess.retain(|id| id != session_id);
                }
            }
        }
    }
}
