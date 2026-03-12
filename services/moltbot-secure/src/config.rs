//! Configuration management for Moltbot Secure

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub bind_address: String,
    pub port: u16,
    pub moltbot_url: String,
    pub moltbot_api_key: Option<String>,
    pub config_path: PathBuf,
    
    // Security settings
    pub encryption: EncryptionConfig,
    pub auth: AuthConfig,
    pub session: SessionConfig,
    
    // Key storage
    pub key_storage_path: PathBuf,
    pub enable_key_rotation: bool,
    pub key_rotation_interval_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub algorithm: String, // "hybrid-pqc" or "aes-gcm"
    pub use_pqc: bool,
    pub use_hybrid_kem: bool, // ML-KEM + HQC hybrid
    pub symmetric_algorithm: String, // "aes-256-gcm" or "chacha20-poly1305"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,
    pub password_hash_algorithm: String, // "argon2" or "pbkdf2"
    pub require_mfa: bool,
    pub max_login_attempts: u32,
    pub lockout_duration_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub session_timeout_minutes: u64,
    pub max_sessions_per_user: u32,
    pub enable_session_encryption: bool,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Try to load from config file, or use defaults
        let config_path = Self::find_config_file()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            let mut config: Config = toml::from_str(&content)
                .context("Failed to parse config file")?;
            config.config_path = config_path;
            Ok(config)
        } else {
            // Create default config
            let default_config = Self::default_config(&config_path);
            // Save default config
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)
                    .context("Failed to create config directory")?;
            }
            let toml_content = toml::to_string_pretty(&default_config)
                .context("Failed to serialize default config")?;
            fs::write(&config_path, toml_content)
                .context("Failed to write default config")?;
            Ok(default_config)
        }
    }

    fn find_config_file() -> Result<PathBuf> {
        // Check environment variable first
        if let Ok(config_path) = std::env::var("MOLTBOT_SECURE_CONFIG") {
            return Ok(PathBuf::from(config_path));
        }
        
        // Check multiple locations
        let locations = [
            "./moltbot-secure.toml",
            "~/.config/moltbot-secure/config.toml",
            "/etc/moltbot-secure/config.toml",
        ];

        for loc in &locations {
            let expanded = shellexpand::full(loc)
                .map(|s| PathBuf::from(s.as_ref()))
                .ok();
            if let Some(path) = expanded {
                if path.exists() {
                    return Ok(path);
                }
            }
        }

        // Return default location
        Ok(PathBuf::from("./moltbot-secure.toml"))
    }

    fn default_config(config_path: &Path) -> Self {
        let key_storage = config_path.parent()
            .map(|p| p.join("keys"))
            .unwrap_or_else(|| PathBuf::from("./keys"));

        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 8443,
            moltbot_url: "http://localhost:3000".to_string(),
            moltbot_api_key: None,
            config_path: config_path.to_path_buf(),
            encryption: EncryptionConfig {
                algorithm: "hybrid-pqc".to_string(),
                use_pqc: true,
                use_hybrid_kem: true,
                symmetric_algorithm: "aes-256-gcm".to_string(),
            },
            auth: AuthConfig {
                jwt_secret: Self::generate_random_secret(),
                jwt_expiry_hours: 24,
                password_hash_algorithm: "argon2".to_string(),
                require_mfa: false,
                max_login_attempts: 5,
                lockout_duration_minutes: 15,
            },
            session: SessionConfig {
                session_timeout_minutes: 60,
                max_sessions_per_user: 10,
                enable_session_encryption: true,
            },
            key_storage_path: key_storage,
            enable_key_rotation: true,
            key_rotation_interval_hours: 24,
        }
    }

    fn generate_random_secret() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        hex::encode(bytes)
    }
}
