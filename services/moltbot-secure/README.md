# Moltbot Secure - Encrypted Wrapper Layer

A secure, encrypted proxy layer for Moltbot that provides end-to-end encryption, secure key management, and protection against the security vulnerabilities found in exposed Moltbot instances.

## Overview

Moltbot Secure wraps Moltbot with multiple layers of security:

- **Post-Quantum Hybrid Encryption**: ML-KEM-1024 + HQC-256 key exchange with AES-256-GCM or ChaCha20-Poly1305 symmetric encryption
- **PQC Signatures**: ML-DSA-87 + SLH-DSA-256s hybrid signatures for message authentication
- **Secure Key Management**: Automatic key rotation, secure key exchange protocol
- **Session Management**: Encrypted sessions with timeout and activity tracking
- **Authentication**: Argon2/PBKDF2 password hashing, JWT tokens, account lockout protection
- **Access Control**: Per-user session limits, encrypted credential storage

## Security Features

### Protection Against Known Vulnerabilities

Moltbot Secure addresses the security issues found in exposed Moltbot instances:

1. **Credential Leakage**: All API keys and credentials are encrypted at rest and in transit
2. **Unauthorized Access**: Authentication required for all operations, with account lockout protection
3. **Conversation History Exposure**: All messages are encrypted end-to-end
4. **Control Panel Exposure**: No exposed control panels - all access via authenticated API
5. **Command Injection**: Input validation and encryption prevent injection attacks

### Encryption Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Client Application                    │
└───────────────────────┬───────────────────────────────────┘
                        │ Encrypted (Hybrid PQC)
                        ▼
┌─────────────────────────────────────────────────────────┐
│              Moltbot Secure Proxy Layer                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   Auth &     │  │   Key        │  │   Session    │ │
│  │   Access     │  │   Exchange   │  │   Manager    │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│                                                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │         Encryption Layer (PQC Hybrid)            │  │
│  │  ML-KEM-1024 + HQC-256 → AES-256-GCM/ChaCha20   │  │
│  │  ML-DSA-87 + SLH-DSA-256s Signatures            │  │
│  └──────────────────────────────────────────────────┘  │
└───────────────────────┬───────────────────────────────────┘
                        │ HTTPS (to Moltbot)
                        ▼
┌─────────────────────────────────────────────────────────┐
│                    Moltbot Instance                     │
└─────────────────────────────────────────────────────────┘
```

## Installation

### Prerequisites

- Rust 1.70+ (for building from source)
- Moltbot instance running and accessible
- Network access to Moltbot API

### Build from Source

```bash
cd services/moltbot-secure
cargo build --release
```

### Configuration

Create a configuration file `moltbot-secure.toml`:

```toml
bind_address = "127.0.0.1"
port = 8443
moltbot_url = "http://localhost:3000"
moltbot_api_key = "your-moltbot-api-key"  # Optional

[encryption]
algorithm = "hybrid-pqc"
use_pqc = true
use_hybrid_kem = true
symmetric_algorithm = "aes-256-gcm"  # or "chacha20-poly1305"

[auth]
jwt_secret = "auto-generated-on-first-run"
jwt_expiry_hours = 24
password_hash_algorithm = "argon2"  # or "pbkdf2"
require_mfa = false
max_login_attempts = 5
lockout_duration_minutes = 15

[session]
session_timeout_minutes = 60
max_sessions_per_user = 10
enable_session_encryption = true

key_storage_path = "./keys"
enable_key_rotation = true
key_rotation_interval_hours = 24
```

## Usage

### Starting the Service

```bash
./target/release/moltbot-secure
```

The service will:
- Create default configuration if none exists
- Initialize key storage directory
- Start listening on configured address/port

### API Endpoints

#### Health Check
```bash
GET /health
```

#### Register User
```bash
POST /api/v1/auth/register
Content-Type: application/json

{
  "username": "alice",
  "password": "secure-password"
}
```

#### Login
```bash
POST /api/v1/auth/login
Content-Type: application/json

{
  "username": "alice",
  "password": "secure-password"
}

Response:
{
  "success": true,
  "token": "auth-token-here",
  "expires_at": 1234567890,
  "user_id": "user-uuid"
}
```

#### Create Session
```bash
POST /api/v1/session/create
Content-Type: application/json
Authorization: Bearer <token>

{
  "user_id": "user-uuid",
  "token": "auth-token"
}

Response:
{
  "success": true,
  "session_id": "session-uuid",
  "expires_at": 1234567890,
  "encryption_enabled": true
}
```

#### Key Exchange
```bash
POST /api/v1/keys/exchange
Content-Type: application/json

{
  "client_id": "client-uuid",
  "client_pubkey": [/* public key bytes */],
  "session_id": "session-uuid"
}

Response:
{
  "server_pubkey": [/* server public key */],
  "session_key": {/* encrypted session key */},
  "key_id": "key-uuid",
  "expires_at": 1234567890
}
```

#### Send Encrypted Message
```bash
POST /api/v1/proxy/message
Content-Type: application/json

{
  "session_id": "session-uuid",
  "encrypted_message": {
    "version": 1,
    "kem_ciphertext": [/* KEM ciphertext */],
    "symmetric_ciphertext": [/* encrypted message */],
    "nonce": [/* nonce */],
    "signature": {/* PQC signatures */},
    "metadata": {/* message metadata */}
  },
  "message_type": "text"
}

Response:
{
  "success": true,
  "encrypted_response": {/* encrypted response from Moltbot */},
  "error": null
}
```

## Client Integration

### Example Client Flow

1. **Register/Login**: Authenticate and receive JWT token
2. **Create Session**: Establish encrypted session
3. **Key Exchange**: Exchange encryption keys with server
4. **Encrypt Messages**: Encrypt messages using session keys
5. **Send Messages**: Send encrypted messages to proxy
6. **Decrypt Responses**: Decrypt responses from proxy

### Encryption Process

1. Client generates ephemeral keypair
2. Client performs hybrid KEM encapsulation with server's public key
3. Client derives symmetric key from KEM shared secret
4. Client encrypts message with symmetric cipher (AES-256-GCM or ChaCha20-Poly1305)
5. Client optionally signs encrypted message with PQC signatures
6. Client sends encrypted message to proxy
7. Proxy decrypts, forwards to Moltbot, encrypts response, returns to client

## Security Considerations

### Key Management

- Keys are stored encrypted at rest
- Automatic key rotation prevents long-term key compromise
- Session keys are ephemeral and expire automatically
- Key exchange uses post-quantum cryptography

### Authentication

- Passwords are hashed with Argon2 (recommended) or PBKDF2
- Account lockout after failed login attempts
- JWT tokens with expiration
- Optional MFA support (planned)

### Encryption

- Hybrid post-quantum + classical cryptography
- Forward secrecy through ephemeral keys
- Message authentication via PQC signatures
- Quantum-resistant algorithms protect against future threats

## Deployment

### Production Deployment

1. **Generate Strong Secrets**: Update `jwt_secret` in config with a strong random value
2. **Use HTTPS**: Deploy behind reverse proxy (nginx/traefik) with TLS
3. **Firewall Rules**: Restrict access to authorized IPs only
4. **Key Storage**: Use secure, encrypted storage for keys directory
5. **Monitoring**: Set up logging and monitoring for security events
6. **Backup**: Regularly backup configuration and key storage

### Docker Deployment

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/moltbot-secure /usr/local/bin/
EXPOSE 8443
CMD ["moltbot-secure"]
```

## Development

### Running Tests

```bash
cargo test
```

### Code Structure

- `src/main.rs`: Application entry point and HTTP handlers
- `src/config.rs`: Configuration management
- `src/encryption.rs`: Hybrid PQC encryption implementation
- `src/key_manager.rs`: Key management and exchange
- `src/auth.rs`: Authentication and authorization
- `src/session.rs`: Session management
- `src/proxy.rs`: Moltbot proxy handler

## Contributing

Contributions welcome! Please ensure:

- Code follows Rust best practices
- Security considerations are documented
- Tests are included for new features
- Encryption implementations are reviewed

## License

MIT License - See LICENSE file for details

## References

- [Moltbot Security Concerns](https://theregister.com/2026/01/27/clawdbot_moltbot_security_concerns)
- [Post-Quantum Cryptography Standards](https://csrc.nist.gov/projects/post-quantum-cryptography)
- [QuantumAegis Project](../README.md)
