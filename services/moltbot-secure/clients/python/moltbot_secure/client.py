"""
Main client for Moltbot Secure API
"""

import requests
import json
from typing import Optional, Dict, Any
from .encryption import EncryptionKeyPair
from .exceptions import (
    MoltbotSecureError,
    AuthenticationError,
    SessionError,
    ProxyError,
)


class MoltbotSecureClient:
    """Client for interacting with Moltbot Secure API"""

    def __init__(
        self,
        base_url: str = "http://localhost:8443",
        verify_ssl: bool = True,
    ):
        """
        Initialize the client.

        Args:
            base_url: Base URL of the Moltbot Secure API
            verify_ssl: Whether to verify SSL certificates
        """
        self.base_url = base_url.rstrip("/")
        self.verify_ssl = verify_ssl
        self.session = requests.Session()
        self.session.verify = verify_ssl
        self.token: Optional[str] = None
        self.user_id: Optional[str] = None
        self.session_id: Optional[str] = None
        self.client_keypair: Optional[EncryptionKeyPair] = None
        self.server_keypair: Optional[EncryptionKeyPair] = None

    def health_check(self) -> Dict[str, Any]:
        """Check if the service is healthy"""
        try:
            response = self.session.get(f"{self.base_url}/health")
            response.raise_for_status()
            return response.json()
        except requests.RequestException as e:
            raise MoltbotSecureError(f"Health check failed: {e}") from e

    def register(self, username: str, password: str) -> Dict[str, Any]:
        """
        Register a new user.

        Args:
            username: Username for the new account
            password: Password for the new account

        Returns:
            Registration response
        """
        try:
            response = self.session.post(
                f"{self.base_url}/api/v1/auth/register",
                json={"username": username, "password": password},
            )
            response.raise_for_status()
            return response.json()
        except requests.HTTPError as e:
            if e.response.status_code == 400:
                raise AuthenticationError(f"Registration failed: {e.response.text}") from e
            raise AuthenticationError(f"Registration error: {e}") from e
        except requests.RequestException as e:
            raise AuthenticationError(f"Registration request failed: {e}") from e

    def login(self, username: str, password: str) -> Dict[str, Any]:
        """
        Login and get authentication token.

        Args:
            username: Username
            password: Password

        Returns:
            Login response with token and user_id
        """
        try:
            response = self.session.post(
                f"{self.base_url}/api/v1/auth/login",
                json={"username": username, "password": password},
            )
            response.raise_for_status()
            result = response.json()
            
            if result.get("success"):
                self.token = result.get("token")
                self.user_id = result.get("user_id")
                self.session.headers.update({"Authorization": f"Bearer {self.token}"})
            
            return result
        except requests.HTTPError as e:
            if e.response.status_code == 401:
                raise AuthenticationError("Invalid username or password") from e
            raise AuthenticationError(f"Login error: {e}") from e
        except requests.RequestException as e:
            raise AuthenticationError(f"Login request failed: {e}") from e

    def create_session(self) -> Dict[str, Any]:
        """
        Create a new encrypted session.

        Returns:
            Session creation response with session_id
        """
        if not self.token or not self.user_id:
            raise AuthenticationError("Must login before creating session")

        try:
            response = self.session.post(
                f"{self.base_url}/api/v1/session/create",
                json={"user_id": self.user_id, "token": self.token},
            )
            response.raise_for_status()
            result = response.json()
            
            if result.get("success"):
                self.session_id = result.get("session_id")
            
            return result
        except requests.HTTPError as e:
            raise SessionError(f"Session creation failed: {e.response.text}") from e
        except requests.RequestException as e:
            raise SessionError(f"Session creation request failed: {e}") from e

    def exchange_keys(self) -> Dict[str, Any]:
        """
        Exchange encryption keys with the server.

        Returns:
            Key exchange response with server public key
        """
        if not self.session_id:
            raise SessionError("Must create session before key exchange")

        # Generate client keypair if not exists
        if not self.client_keypair:
            self.client_keypair = EncryptionKeyPair.generate()

        try:
            response = self.session.post(
                f"{self.base_url}/api/v1/keys/exchange",
                json={
                    "client_id": self.user_id or "client",
                    "client_pubkey": self.client_keypair.public_key_bytes().tolist(),
                    "session_id": self.session_id,
                },
            )
            response.raise_for_status()
            return response.json()
        except requests.RequestException as e:
            raise MoltbotSecureError(f"Key exchange failed: {e}") from e

    def send_message(
        self,
        message: str,
        message_type: str = "text",
    ) -> Dict[str, Any]:
        """
        Send an encrypted message to Moltbot.

        Args:
            message: Message text to send
            message_type: Type of message (text, command, file, etc.)

        Returns:
            Encrypted response from Moltbot
        """
        if not self.session_id:
            raise SessionError("Must create session before sending messages")
        if not self.client_keypair:
            raise SessionError("Must exchange keys before sending messages")

        # Encrypt the message
        from .encryption import encrypt_message
        
        message_bytes = message.encode("utf-8")
        encrypted_msg = encrypt_message(
            message_bytes,
            self.server_keypair or self.client_keypair,  # Use server keypair if available
            self.client_keypair,
            "aes-256-gcm",
        )

        try:
            response = self.session.post(
                f"{self.base_url}/api/v1/proxy/message",
                json={
                    "session_id": self.session_id,
                    "encrypted_message": encrypted_msg.to_dict(),
                    "message_type": message_type,
                },
            )
            response.raise_for_status()
            result = response.json()
            
            # Decrypt the response if present
            if result.get("success") and result.get("encrypted_response"):
                from .encryption import decrypt_message
                encrypted_response = result["encrypted_response"]
                decrypted = decrypt_message(
                    encrypted_response,
                    self.client_keypair,
                    None,
                )
                result["decrypted_response"] = json.loads(decrypted.decode("utf-8"))
            
            return result
        except requests.RequestException as e:
            raise ProxyError(f"Failed to send message: {e}") from e

    def close(self):
        """Close the client session"""
        self.session.close()
        self.token = None
        self.user_id = None
        self.session_id = None

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()
