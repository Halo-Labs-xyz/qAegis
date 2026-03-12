/**
 * Example usage of Moltbot Secure JavaScript client
 */

const { MoltbotSecureClient } = require("../clients/javascript/dist");

async function main() {
  // Initialize client
  const client = new MoltbotSecureClient("http://localhost:8443");

  try {
    // Health check
    console.log("Checking service health...");
    const health = await client.healthCheck();
    console.log(`Service status: ${health.status}`);
    console.log(`Version: ${health.version}\n`);

    // Register a new user
    console.log("Registering new user...");
    try {
      await client.register("testuser", "testpass123");
      console.log("✅ User registered successfully\n");
    } catch (error) {
      if (error.message.includes("already exists")) {
        console.log("ℹ️  User already exists, continuing...\n");
      } else {
        throw error;
      }
    }

    // Login
    console.log("Logging in...");
    const loginResult = await client.login("testuser", "testpass123");
    console.log("✅ Login successful!");
    console.log(`   User ID: ${loginResult.user_id}`);
    console.log(`   Token expires at: ${loginResult.expires_at}\n`);

    // Create session
    console.log("Creating session...");
    const sessionResult = await client.createSession();
    console.log("✅ Session created!");
    console.log(`   Session ID: ${sessionResult.session_id}`);
    console.log(`   Encryption enabled: ${sessionResult.encryption_enabled}\n`);

    // Exchange keys
    console.log("Exchanging encryption keys...");
    const keyResult = await client.exchangeKeys();
    console.log("✅ Keys exchanged!");
    console.log(`   Key ID: ${keyResult.key_id}\n`);

    // Send a message
    console.log("Sending encrypted message...");
    const messageResponse = await client.sendMessage("Hello from JavaScript client!");
    console.log("✅ Message sent!");
    if (messageResponse.decrypted_response) {
      console.log(`   Response: ${JSON.stringify(messageResponse.decrypted_response)}`);
    }
    console.log();

    console.log("✅ All operations completed successfully!");
  } catch (error) {
    console.error(`❌ Error: ${error.message}`);
    process.exit(1);
  } finally {
    client.close();
  }
}

main();
