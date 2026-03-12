//! Key Management and Exchange Protocol
//!
//! Manages encryption keys, key rotation, and secure key exchange
//! between clients and the proxy server.

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

use crate::config::Config;
use crate::encryption::{EncryptionKeyPair, encrypt_message, EncryptedMessage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyExchangeRequest {
    pub client_id: String,
    pub client_pubkey: Vec<u8>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyExchangeResponse {
    pub server_pubkey: Vec<u8>,
    pub session_key: EncryptedMessage, // Server's public key encrypted with client's public key
    pub key_id: String,
    pub expires_at: i64,
}

#[derive(Debug, Clone)]
struct StoredKeyPair {
    keypair: EncryptionKeyPair,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    key_id: String,
    user_id: Option<String>,
}

pub struct KeyManager {
    config: Arc<Config>,
    // Active keypairs indexed by key_id
    keypairs: Arc<RwLock<HashMap<String, StoredKeyPair>>>,
    // User keypairs indexed by user_id
    user_keys: Arc<RwLock<HashMap<String, Vec<String>>>>, // user_id -> [key_ids]
    // Session keys indexed by session_id
    session_keys: Arc<RwLock<HashMap<String, EncryptionKeyPair>>>,
}

impl KeyManager {
    pub fn new(config: &Arc<Config>) -> Result<Self> {
        // Ensure key storage directory exists
        std::fs::create_dir_all(&config.key_storage_path)
            .context("Failed to create key storage directory")?;

        Ok(Self {
            config: config.clone(),
            keypairs: Arc::new(RwLock::new(HashMap::new())),
            user_keys: Arc::new(RwLock::new(HashMap::new())),
            session_keys: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Generate a new keypair for a user
    pub async fn generate_user_keypair(&self, user_id: &str) -> Result<String> {
        let keypair = EncryptionKeyPair::generate();
        let key_id = uuid::Uuid::new_v4().to_string();
        
        let expires_at = if self.config.enable_key_rotation {
            Utc::now() + Duration::hours(self.config.key_rotation_interval_hours as i64)
        } else {
            Utc::now() + Duration::days(365)
        };

        let stored = StoredKeyPair {
            keypair,
            created_at: Utc::now(),
            expires_at,
            key_id: key_id.clone(),
            user_id: Some(user_id.to_string()),
        };

        // Store keypair
        self.keypairs.write().await.insert(key_id.clone(), stored);

        // Associate with user
        self.user_keys.write().await
            .entry(user_id.to_string())
            .or_insert_with(Vec::new)
            .push(key_id.clone());

        Ok(key_id)
    }

    /// Exchange keys with a client (ECDH-like key exchange)
    pub async fn exchange_keys(&self, request: &serde_json::Value) -> Result<serde_json::Value> {
        let req: KeyExchangeRequest = serde_json::from_value(request.clone())
            .context("Invalid key exchange request")?;

        // Generate or retrieve server keypair for this session
        let session_id = req.session_id.clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        
        let server_keypair = if let Some(existing) = self.session_keys.read().await.get(&session_id) {
            existing.clone()
        } else {
            let new_keypair = EncryptionKeyPair::generate();
            self.session_keys.write().await.insert(session_id.clone(), new_keypair.clone());
            new_keypair
        };

        // Parse client public key (simplified - in production, properly deserialize)
        // For now, we'll generate a temporary keypair for the client
        let client_keypair = EncryptionKeyPair::generate();

        // Encrypt server's public key with client's public key
        // In a real implementation, we'd use proper key exchange protocol
        let server_pubkey_bytes = server_keypair.public_key_bytes();
        
        // Create a session key encrypted message
        // This is a simplified version - in production, use proper ECDH/PQC key exchange
        let session_key_message = serde_json::json!({
            "server_pubkey": hex::encode(&server_pubkey_bytes),
            "session_id": session_id,
            "algorithm": "hybrid-pqc",
        });
        
        let session_key_bytes = serde_json::to_vec(&session_key_message)?;
        let encrypted_session_key = encrypt_message(
            &session_key_bytes,
            &client_keypair,
            Some(&server_keypair),
            &self.config.encryption.symmetric_algorithm,
        )?;

        let expires_at = Utc::now() + chrono::Duration::hours(
            self.config.session.session_timeout_minutes as i64 / 60
        );

        let response = KeyExchangeResponse {
            server_pubkey: server_pubkey_bytes,
            session_key: encrypted_session_key,
            key_id: session_id.clone(),
            expires_at: expires_at.timestamp(),
        };

        Ok(serde_json::to_value(response)?)
    }

    /// Get keypair by key_id
    pub async fn get_keypair(&self, key_id: &str) -> Option<EncryptionKeyPair> {
        self.keypairs.read().await
            .get(key_id)
            .map(|stored| stored.keypair.clone())
    }

    /// Get session keypair
    pub async fn get_session_keypair(&self, session_id: &str) -> Option<EncryptionKeyPair> {
        self.session_keys.read().await
            .get(session_id)
            .cloned()
    }

    /// Rotate keys for a user
    pub async fn rotate_user_keys(&self, user_id: &str) -> Result<String> {
        // Generate new keypair
        let new_key_id = self.generate_user_keypair(user_id).await?;
        
        // Mark old keys for expiration (they'll be cleaned up later)
        // In production, implement proper key rotation with grace period
        
        Ok(new_key_id)
    }

    /// Clean up expired keys
    pub async fn cleanup_expired_keys(&self) {
        let now = Utc::now();
        let mut keypairs = self.keypairs.write().await;
        
        keypairs.retain(|_, stored| stored.expires_at > now);
    }
}
