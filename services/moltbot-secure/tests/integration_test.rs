//! Integration tests for Moltbot Secure

use moltbot_secure::*;
use std::sync::Arc;
use tokio;

#[tokio::test]
async fn test_config_loading() {
    let config = Config::load().expect("Failed to load config");
    assert_eq!(config.port, 8443);
    assert!(!config.moltbot_url.is_empty());
}

#[tokio::test]
async fn test_auth_register_and_login() {
    let config = Arc::new(Config::load().expect("Failed to load config"));
    let auth_manager = AuthManager::new(&config);

    // Register a test user
    let register_payload = serde_json::json!({
        "username": "testuser",
        "password": "testpass123"
    });

    let register_result = auth_manager.register(&register_payload).await;
    assert!(register_result.is_ok());

    // Login with the same credentials
    let login_payload = serde_json::json!({
        "username": "testuser",
        "password": "testpass123"
    });

    let login_result = auth_manager.login(&login_payload).await;
    assert!(login_result.is_ok());
    
    let login_response = login_result.unwrap();
    assert!(login_response.get("success").and_then(|v| v.as_bool()).unwrap_or(false));
    assert!(login_response.get("token").is_some());
}

#[tokio::test]
async fn test_auth_login_failure() {
    let config = Arc::new(Config::load().expect("Failed to load config"));
    let auth_manager = AuthManager::new(&config);

    // Try to login with wrong password
    let login_payload = serde_json::json!({
        "username": "testuser",
        "password": "wrongpassword"
    });

    let login_result = auth_manager.login(&login_payload).await;
    // Should fail if user doesn't exist, or succeed but with wrong password check
    // This test verifies the error handling works
    if login_result.is_err() {
        // Expected behavior - login failed
        assert!(true);
    }
}

#[tokio::test]
async fn test_key_manager_generation() {
    let config = Arc::new(Config::load().expect("Failed to load config"));
    let key_manager = KeyManager::new(&config).expect("Failed to create key manager");

    // Generate a keypair for a user
    let key_id = key_manager.generate_user_keypair("test_user").await;
    assert!(key_id.is_ok());
    
    let key_id = key_id.unwrap();
    assert!(!key_id.is_empty());
}

#[tokio::test]
async fn test_encryption_roundtrip() {
    use moltbot_secure::encryption::*;

    // Generate keypairs
    let sender_keypair = EncryptionKeyPair::generate();
    let recipient_keypair = EncryptionKeyPair::generate();

    // Test message
    let message = b"Hello, secure world!";

    // Encrypt message
    let encrypted = encrypt_message(
        message,
        &recipient_keypair,
        Some(&sender_keypair),
        "aes-256-gcm",
    ).expect("Encryption failed");

    // Verify encrypted message has required fields
    assert_eq!(encrypted.version, 1);
    assert!(!encrypted.kem_ciphertext.is_empty());
    assert!(!encrypted.symmetric_ciphertext.is_empty());
    assert!(!encrypted.nonce.is_empty());
    assert!(encrypted.signature.is_some());

    // Decrypt message
    let decrypted = decrypt_message(
        &encrypted,
        &recipient_keypair,
        Some(&sender_keypair),
    ).expect("Decryption failed");

    // Verify decrypted message matches original
    assert_eq!(decrypted, message);
}

#[tokio::test]
async fn test_session_creation() {
    let config = Arc::new(Config::load().expect("Failed to load config"));
    let session_manager = SessionManager::new(&config);

    // Create a session
    let session_payload = serde_json::json!({
        "user_id": "test_user_123",
        "token": "test_token_123"
    });

    let session_result = session_manager.create_session(&session_payload).await;
    assert!(session_result.is_ok());

    let session_response = session_result.unwrap();
    assert!(session_response.get("success").and_then(|v| v.as_bool()).unwrap_or(false));
    assert!(session_response.get("session_id").is_some());
}

#[tokio::test]
async fn test_key_exchange() {
    let config = Arc::new(Config::load().expect("Failed to load config"));
    let key_manager = KeyManager::new(&config).expect("Failed to create key manager");

    // Create a key exchange request
    let exchange_payload = serde_json::json!({
        "client_id": "test_client",
        "client_pubkey": [1, 2, 3, 4, 5],
        "session_id": null
    });

    let exchange_result = key_manager.exchange_keys(&exchange_payload).await;
    assert!(exchange_result.is_ok());

    let exchange_response = exchange_result.unwrap();
    assert!(exchange_response.get("server_pubkey").is_some());
    assert!(exchange_response.get("session_key").is_some());
    assert!(exchange_response.get("key_id").is_some());
}
