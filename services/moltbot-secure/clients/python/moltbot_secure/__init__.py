"""
Moltbot Secure Python Client Library

A secure client library for interacting with Moltbot Secure API.
Provides encryption, authentication, and secure message handling.
"""

from .client import MoltbotSecureClient
from .encryption import EncryptionKeyPair, encrypt_message, decrypt_message
from .exceptions import (
    MoltbotSecureError,
    AuthenticationError,
    EncryptionError,
    SessionError,
    ProxyError,
)

__version__ = "0.1.0"
__all__ = [
    "MoltbotSecureClient",
    "EncryptionKeyPair",
    "encrypt_message",
    "decrypt_message",
    "MoltbotSecureError",
    "AuthenticationError",
    "EncryptionError",
    "SessionError",
    "ProxyError",
]
