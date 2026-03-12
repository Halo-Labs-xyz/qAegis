//! Secure Proxy Handler for Moltbot
//!
//! Handles encrypted communication between clients and Moltbot,
//! ensuring all data is encrypted end-to-end

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use reqwest::Client;

use crate::config::Config;
use crate::encryption::{EncryptedMessage, encrypt_message, decrypt_message};
use crate::key_manager::KeyManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyMessageRequest {
    pub session_id: String,
    pub encrypted_message: EncryptedMessage,
    pub message_type: String, // "text", "command", "file", etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyMessageResponse {
    pub success: bool,
    pub encrypted_response: Option<EncryptedMessage>,
    pub error: Option<String>,
}

pub struct ProxyHandler {
    config: Arc<Config>,
    key_manager: Arc<KeyManager>,
    http_client: Client,
}

impl ProxyHandler {
    pub fn new(config: &Arc<Config>, key_manager: Arc<KeyManager>) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            key_manager,
            http_client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()?,
        })
    }

    /// Handle an encrypted message from client, proxy to Moltbot, return encrypted response
    pub async fn handle_message(&self, payload: &serde_json::Value) -> Result<serde_json::Value> {
        let request: ProxyMessageRequest = serde_json::from_value(payload.clone())
            .context("Invalid proxy message request")?;

        // Get session keypair
        let session_keypair = self.key_manager.get_session_keypair(&request.session_id).await
            .context("Session not found or invalid")?;

        // Decrypt the client's message
        let decrypted_message = decrypt_message(
            &request.encrypted_message,
            &session_keypair,
            None, // No signature verification for now
        )?;

        // Parse the decrypted message
        let message_data: serde_json::Value = serde_json::from_slice(&decrypted_message)
            .context("Failed to parse decrypted message")?;

        // Forward to Moltbot (unencrypted on this side, but encrypted in transit via HTTPS)
        let moltbot_response = self.forward_to_moltbot(&message_data).await?;

        // Encrypt the response
        let encrypted_response = encrypt_message(
            &serde_json::to_vec(&moltbot_response)?,
            &session_keypair,
            Some(&session_keypair), // Sign with session keypair
            &self.config.encryption.symmetric_algorithm,
        )?;

        let response = ProxyMessageResponse {
            success: true,
            encrypted_response: Some(encrypted_response),
            error: None,
        };

        Ok(serde_json::to_value(response)?)
    }

    /// Forward message to Moltbot API
    async fn forward_to_moltbot(&self, message: &serde_json::Value) -> Result<serde_json::Value> {
        let url = format!("{}/api/message", self.config.moltbot_url);
        
        let mut request_builder = self.http_client.post(&url);

        // Add API key if configured
        if let Some(api_key) = &self.config.moltbot_api_key {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        // Add standard headers
        request_builder = request_builder
            .header("Content-Type", "application/json")
            .header("User-Agent", "moltbot-secure/0.1.0");

        let response = request_builder
            .json(message)
            .send()
            .await
            .context("Failed to send request to Moltbot")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("Moltbot API error: {} - {}", status, error_text));
        }

        let response_json: serde_json::Value = response.json().await
            .context("Failed to parse Moltbot response")?;

        Ok(response_json)
    }
}
