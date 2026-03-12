# Moltbot Secure Python Client

Python client library for Moltbot Secure API.

## Installation

```bash
pip install -e .
```

Or from PyPI (when published):
```bash
pip install moltbot-secure
```

## Usage

```python
from moltbot_secure import MoltbotSecureClient

# Initialize client
client = MoltbotSecureClient(base_url="https://your-server.com")

# Register a new user
client.register("username", "password")

# Login
client.login("username", "password")

# Create a session
client.create_session()

# Exchange keys
client.exchange_keys()

# Send an encrypted message
response = client.send_message("Hello, Moltbot!")
print(response)

# Close the client
client.close()
```

## Context Manager

```python
with MoltbotSecureClient() as client:
    client.login("username", "password")
    client.create_session()
    client.exchange_keys()
    response = client.send_message("Hello!")
```

## Error Handling

```python
from moltbot_secure import (
    MoltbotSecureError,
    AuthenticationError,
    SessionError,
    ProxyError,
)

try:
    client.login("username", "password")
except AuthenticationError as e:
    print(f"Authentication failed: {e}")
except MoltbotSecureError as e:
    print(f"Error: {e}")
```

## Examples

See `examples/` directory for more examples.
