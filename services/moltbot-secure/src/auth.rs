//! Authentication and Authorization Manager
//!
//! Handles user registration, login, JWT tokens, and access control

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use sha2::{Sha256, Digest};
use hex;

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub failed_login_attempts: u32,
    pub locked_until: Option<DateTime<Utc>>,
    pub mfa_enabled: bool,
    pub mfa_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub expires_at: i64,
    pub user_id: String,
}

pub struct AuthManager {
    config: Arc<Config>,
    users: Arc<RwLock<HashMap<String, User>>>, // username -> User
    tokens: Arc<RwLock<HashMap<String, AuthToken>>>, // token -> AuthToken
}

impl AuthManager {
    pub fn new(config: &Arc<Config>) -> Self {
        Self {
            config: config.clone(),
            users: Arc::new(RwLock::new(HashMap::new())),
            tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new user
    pub async fn register(&self, payload: &serde_json::Value) -> Result<serde_json::Value> {
        let username = payload.get("username")
            .and_then(|v| v.as_str())
            .context("Missing username")?;
        let password = payload.get("password")
            .and_then(|v| v.as_str())
            .context("Missing password")?;

        // Check if user already exists
        if self.users.read().await.contains_key(username) {
            return Err(anyhow::anyhow!("Username already exists"));
        }

        // Hash password
        let password_hash = self.hash_password(password)?;

        // Create user
        let user = User {
            user_id: Uuid::new_v4().to_string(),
            username: username.to_string(),
            password_hash,
            created_at: Utc::now(),
            last_login: None,
            failed_login_attempts: 0,
            locked_until: None,
            mfa_enabled: false,
            mfa_secret: None,
        };

        self.users.write().await.insert(username.to_string(), user.clone());

        Ok(serde_json::json!({
            "success": true,
            "user_id": user.user_id,
            "message": "User registered successfully"
        }))
    }

    /// Login user
    pub async fn login(&self, payload: &serde_json::Value) -> Result<serde_json::Value> {
        let username = payload.get("username")
            .and_then(|v| v.as_str())
            .context("Missing username")?;
        let password = payload.get("password")
            .and_then(|v| v.as_str())
            .context("Missing password")?;

        let mut users = self.users.write().await;
        let user = users.get_mut(username)
            .ok_or_else(|| anyhow::anyhow!("Invalid username or password"))?;

        // Check if account is locked
        if let Some(locked_until) = user.locked_until {
            if locked_until > Utc::now() {
                return Err(anyhow::anyhow!("Account is locked. Try again later."));
            } else {
                // Unlock account
                user.locked_until = None;
                user.failed_login_attempts = 0;
            }
        }

        // Verify password
        if !self.verify_password(password, &user.password_hash)? {
            user.failed_login_attempts += 1;
            
            if user.failed_login_attempts >= self.config.auth.max_login_attempts {
                user.locked_until = Some(
                    Utc::now() + Duration::minutes(self.config.auth.lockout_duration_minutes as i64)
                );
                return Err(anyhow::anyhow!("Too many failed login attempts. Account locked."));
            }
            
            return Err(anyhow::anyhow!("Invalid username or password"));
        }

        // Reset failed attempts
        user.failed_login_attempts = 0;
        user.last_login = Some(Utc::now());

        // Generate token
        let token = self.generate_token(&user.user_id)?;
        let expires_at = Utc::now() + Duration::hours(self.config.auth.jwt_expiry_hours as i64);

        let auth_token = AuthToken {
            token: token.clone(),
            expires_at: expires_at.timestamp(),
            user_id: user.user_id.clone(),
        };

        self.tokens.write().await.insert(token.clone(), auth_token);

        Ok(serde_json::json!({
            "success": true,
            "token": token,
            "expires_at": expires_at.timestamp(),
            "user_id": user.user_id
        }))
    }

    /// Verify authentication token
    pub async fn verify_token(&self, token: &str) -> Result<String> {
        let tokens = self.tokens.read().await;
        let auth_token = tokens.get(token)
            .ok_or_else(|| anyhow::anyhow!("Invalid token"))?;

        // Check expiration
        if auth_token.expires_at < Utc::now().timestamp() {
            return Err(anyhow::anyhow!("Token expired"));
        }

        Ok(auth_token.user_id.clone())
    }

    fn hash_password(&self, password: &str) -> Result<String> {
        match self.config.auth.password_hash_algorithm.as_str() {
            "argon2" => {
                use argon2::{Argon2, PasswordHasher, password_hash::{rand_core::OsRng, SaltString}};
                let salt = SaltString::generate(&mut OsRng);
                let argon2 = Argon2::default();
                let password_hash = argon2.hash_password(password.as_bytes(), &salt)
                    .map_err(|e| anyhow::anyhow!("Argon2 hashing failed: {:?}", e))?;
                Ok(password_hash.to_string())
            }
            "pbkdf2" => {
                use pbkdf2::pbkdf2_hmac;
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let salt: [u8; 16] = rng.gen();
                let mut output = [0u8; 32];
                pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, 100000, &mut output);
                Ok(format!("{}:{}", hex::encode(salt), hex::encode(output)))
            }
            _ => Err(anyhow::anyhow!("Unsupported password hash algorithm")),
        }
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        match self.config.auth.password_hash_algorithm.as_str() {
            "argon2" => {
                use argon2::{Argon2, PasswordVerifier};
                let parsed_hash = argon2::password_hash::PasswordHash::new(hash)
                    .map_err(|e| anyhow::anyhow!("Invalid password hash format: {:?}", e))?;
                let argon2 = Argon2::default();
                Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
            }
            "pbkdf2" => {
                let parts: Vec<&str> = hash.split(':').collect();
                if parts.len() != 2 {
                    return Ok(false);
                }
                let salt = hex::decode(parts[0])
                    .context("Invalid salt format")?;
                let expected_hash = hex::decode(parts[1])
                    .context("Invalid hash format")?;
                
                use pbkdf2::pbkdf2_hmac;
                let mut computed_hash = [0u8; 32];
                pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, 100000, &mut computed_hash);
                
                Ok(computed_hash.as_slice() == expected_hash.as_slice())
            }
            _ => Err(anyhow::anyhow!("Unsupported password hash algorithm")),
        }
    }

    fn generate_token(&self, user_id: &str) -> Result<String> {
        // Simple token generation - in production, use proper JWT
        let mut hasher = Sha256::new();
        hasher.update(user_id);
        hasher.update(&self.config.auth.jwt_secret);
        hasher.update(&Utc::now().timestamp().to_string());
        let hash = hasher.finalize();
        Ok(hex::encode(hash))
    }
}
