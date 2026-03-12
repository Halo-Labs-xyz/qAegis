# Quick Start Guide - Moltbot Secure

## Prerequisites

1. **Rust installed**: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. **Moltbot instance**: Ensure Moltbot is running and accessible
3. **Network access**: Ability to connect to Moltbot API

## Installation Steps

### 1. Build the Service

```bash
cd services/moltbot-secure
cargo build --release
```

### 2. Configure

Copy the example configuration:

```bash
cp moltbot-secure.toml.example moltbot-secure.toml
```

Edit `moltbot-secure.toml` and set:
- `moltbot_url`: Your Moltbot instance URL (e.g., `http://localhost:3000`)
- `moltbot_api_key`: If your Moltbot instance requires an API key

### 3. Run the Service

```bash
./target/release/moltbot-secure
```

The service will start on `http://127.0.0.1:8443` by default.

### 4. Test the Service

```bash
# Health check
curl http://localhost:8443/health

# Register a user
curl -X POST http://localhost:8443/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"testpass123"}'

# Login
curl -X POST http://localhost:8443/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"testpass123"}'
```

## Integration Example

### Python Client Example

```python
import requests
import json
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
from cryptography.hazmat.backends import default_backend
import base64

BASE_URL = "http://localhost:8443"

# 1. Register/Login
def login(username, password):
    response = requests.post(
        f"{BASE_URL}/api/v1/auth/login",
        json={"username": username, "password": password}
    )
    return response.json()

# 2. Create Session
def create_session(user_id, token):
    response = requests.post(
        f"{BASE_URL}/api/v1/session/create",
        json={"user_id": user_id, "token": token},
        headers={"Authorization": f"Bearer {token}"}
    )
    return response.json()

# 3. Key Exchange (simplified - in production use proper PQC libraries)
def exchange_keys(client_id, session_id):
    # Generate client keypair (simplified)
    client_pubkey = b"mock-public-key"  # In production, use real PQC keypair
    
    response = requests.post(
        f"{BASE_URL}/api/v1/keys/exchange",
        json={
            "client_id": client_id,
            "client_pubkey": list(client_pubkey),
            "session_id": session_id
        }
    )
    return response.json()

# 4. Send Encrypted Message
def send_message(session_id, encrypted_message):
    response = requests.post(
        f"{BASE_URL}/api/v1/proxy/message",
        json={
            "session_id": session_id,
            "encrypted_message": encrypted_message,
            "message_type": "text"
        }
    )
    return response.json()

# Usage
if __name__ == "__main__":
    # Login
    auth_result = login("testuser", "testpass123")
    token = auth_result["token"]
    user_id = auth_result["user_id"]
    
    # Create session
    session_result = create_session(user_id, token)
    session_id = session_result["session_id"]
    
    print(f"Session created: {session_id}")
    print("Ready to send encrypted messages!")
```

### JavaScript/TypeScript Client Example

```typescript
const BASE_URL = "http://localhost:8443";

// 1. Login
async function login(username: string, password: string) {
  const response = await fetch(`${BASE_URL}/api/v1/auth/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username, password }),
  });
  return response.json();
}

// 2. Create Session
async function createSession(userId: string, token: string) {
  const response = await fetch(`${BASE_URL}/api/v1/session/create`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "Authorization": `Bearer ${token}`,
    },
    body: JSON.stringify({ user_id: userId, token }),
  });
  return response.json();
}

// Usage
(async () => {
  const auth = await login("testuser", "testpass123");
  const session = await createSession(auth.user_id, auth.token);
  console.log("Session created:", session.session_id);
})();
```

## Security Best Practices

1. **Use HTTPS in Production**: Deploy behind a reverse proxy (nginx/traefik) with TLS
2. **Strong Passwords**: Enforce strong password policies
3. **Key Rotation**: Enable automatic key rotation in config
4. **Monitor Logs**: Set up logging and alerting for security events
5. **Network Isolation**: Restrict access to authorized IPs only
6. **Regular Updates**: Keep dependencies updated for security patches

## Troubleshooting

### Service Won't Start

- Check if port 8443 is already in use: `lsof -i :8443`
- Verify configuration file syntax: `toml` format must be valid
- Check logs for error messages

### Connection Errors

- Verify Moltbot is running: `curl http://localhost:3000/health` (adjust URL)
- Check firewall rules
- Verify `moltbot_url` in config matches your Moltbot instance

### Authentication Issues

- Ensure passwords match exactly (case-sensitive)
- Check account lockout status (wait for lockout duration)
- Verify JWT token hasn't expired

## Next Steps

- Read the full [README.md](README.md) for detailed documentation
- Review [Security Considerations](README.md#security-considerations)
- Check [Deployment Guide](README.md#deployment) for production setup
