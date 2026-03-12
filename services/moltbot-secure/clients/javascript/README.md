# Moltbot Secure JavaScript/TypeScript Client

JavaScript/TypeScript client library for Moltbot Secure API.

## Installation

```bash
npm install moltbot-secure
```

Or with yarn:
```bash
yarn add moltbot-secure
```

## Usage

### TypeScript/JavaScript

```typescript
import { MoltbotSecureClient } from "moltbot-secure";

// Initialize client
const client = new MoltbotSecureClient("https://your-server.com");

// Register a new user
await client.register("username", "password");

// Login
await client.login("username", "password");

// Create a session
await client.createSession();

// Exchange keys
await client.exchangeKeys();

// Send an encrypted message
const response = await client.sendMessage("Hello, Moltbot!");
console.log(response);

// Close the client
client.close();
```

### Error Handling

```typescript
import {
  MoltbotSecureError,
  AuthenticationError,
  SessionError,
  ProxyError,
} from "moltbot-secure";

try {
  await client.login("username", "password");
} catch (error) {
  if (error instanceof AuthenticationError) {
    console.error("Authentication failed:", error.message);
  } else if (error instanceof MoltbotSecureError) {
    console.error("Error:", error.message);
  }
}
```

## Examples

See `examples/` directory for more examples.
