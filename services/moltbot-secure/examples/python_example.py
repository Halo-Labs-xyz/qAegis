#!/usr/bin/env python3
"""
Example usage of Moltbot Secure Python client
"""

import sys
import os

# Add the client to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "clients", "python"))

from moltbot_secure import MoltbotSecureClient, AuthenticationError


def main():
    # Initialize client
    client = MoltbotSecureClient(base_url="http://localhost:8443")

    try:
        # Health check
        print("Checking service health...")
        health = client.health_check()
        print(f"Service status: {health['status']}")
        print(f"Version: {health['version']}\n")

        # Register a new user
        print("Registering new user...")
        try:
            client.register("testuser", "testpass123")
            print("✅ User registered successfully\n")
        except AuthenticationError as e:
            if "already exists" in str(e):
                print("ℹ️  User already exists, continuing...\n")
            else:
                raise

        # Login
        print("Logging in...")
        login_result = client.login("testuser", "testpass123")
        print(f"✅ Login successful!")
        print(f"   User ID: {login_result['user_id']}")
        print(f"   Token expires at: {login_result['expires_at']}\n")

        # Create session
        print("Creating session...")
        session_result = client.create_session()
        print(f"✅ Session created!")
        print(f"   Session ID: {session_result['session_id']}")
        print(f"   Encryption enabled: {session_result['encryption_enabled']}\n")

        # Exchange keys
        print("Exchanging encryption keys...")
        key_result = client.exchange_keys()
        print(f"✅ Keys exchanged!")
        print(f"   Key ID: {key_result['key_id']}\n")

        # Send a message
        print("Sending encrypted message...")
        message_response = client.send_message("Hello from Python client!")
        print(f"✅ Message sent!")
        if message_response.get("decrypted_response"):
            print(f"   Response: {message_response['decrypted_response']}")
        print()

        print("✅ All operations completed successfully!")

    except Exception as e:
        print(f"❌ Error: {e}")
        return 1

    finally:
        client.close()

    return 0


if __name__ == "__main__":
    sys.exit(main())
