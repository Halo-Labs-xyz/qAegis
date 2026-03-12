"""
Encryption utilities for Moltbot Secure client
"""

import json
from typing import Dict, Any, Optional
from cryptography.hazmat.primitives.ciphers.aead import AESGCM, ChaCha20Poly1305
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.backends import default_backend
import os
import secrets


class EncryptionKeyPair:
    """Encryption key pair for client-side encryption"""

    def __init__(self):
        """Generate a new key pair"""
        # In a real implementation, this would generate PQC keys
        # For now, we'll use a simplified approach
        self.public_key = secrets.token_bytes(32)
        self.secret_key = secrets.token_bytes(32)

    @classmethod
    def generate(cls) -> "EncryptionKeyPair":
        """Generate a new key pair"""
        return cls()

    def public_key_bytes(self) -> bytes:
        """Get public key as bytes"""
        return self.public_key


def encrypt_message(
    message: bytes,
    recipient_keypair: EncryptionKeyPair,
    signer_keypair: Optional[EncryptionKeyPair] = None,
    algorithm: str = "aes-256-gcm",
) -> Dict[str, Any]:
    """
    Encrypt a message.

    Args:
        message: Message bytes to encrypt
        recipient_keypair: Recipient's key pair
        signer_keypair: Optional signer's key pair
        algorithm: Encryption algorithm (aes-256-gcm or chacha20-poly1305)

    Returns:
        Encrypted message dictionary
    """
    # Generate a symmetric key (simplified - in production use proper KEM)
    symmetric_key = secrets.token_bytes(32)
    nonce = secrets.token_bytes(12)  # 12 bytes for GCM

    # Encrypt with symmetric cipher
    if algorithm == "aes-256-gcm":
        cipher = AESGCM(symmetric_key)
        ciphertext = cipher.encrypt(nonce, message, None)
    elif algorithm == "chacha20-poly1305":
        cipher = ChaCha20Poly1305(symmetric_key)
        ciphertext = cipher.encrypt(nonce, message, None)
    else:
        raise ValueError(f"Unsupported algorithm: {algorithm}")

    # Create encrypted message structure
    encrypted_msg = {
        "version": 1,
        "kem_ciphertext": secrets.token_bytes(100).hex(),  # Mock KEM ciphertext
        "symmetric_ciphertext": ciphertext.hex(),
        "nonce": nonce.hex(),
        "signature": None,
        "metadata": {
            "timestamp": int(__import__("time").time()),
            "message_id": secrets.token_hex(16),
            "algorithm": algorithm,
            "key_id": recipient_keypair.public_key_bytes()[:16].hex(),
        },
    }

    # Add signature if signer provided
    if signer_keypair:
        # In production, use proper PQC signatures
        encrypted_msg["signature"] = {
            "mldsa_sig": secrets.token_bytes(100).hex(),
            "slhdsa_sig": secrets.token_bytes(100).hex(),
        }

    return encrypted_msg


def decrypt_message(
    encrypted_msg: Dict[str, Any],
    recipient_keypair: EncryptionKeyPair,
    verifier_keypair: Optional[EncryptionKeyPair] = None,
) -> bytes:
    """
    Decrypt a message.

    Args:
        encrypted_msg: Encrypted message dictionary
        recipient_keypair: Recipient's key pair
        verifier_keypair: Optional verifier's key pair

    Returns:
        Decrypted message bytes
    """
    # Extract ciphertext and nonce
    ciphertext = bytes.fromhex(encrypted_msg["symmetric_ciphertext"])
    nonce = bytes.fromhex(encrypted_msg["nonce"])
    algorithm = encrypted_msg["metadata"]["algorithm"]

    # Derive symmetric key (simplified - in production use proper KEM decapsulation)
    symmetric_key = secrets.token_bytes(32)

    # Decrypt with symmetric cipher
    if algorithm == "aes-256-gcm":
        cipher = AESGCM(symmetric_key)
        plaintext = cipher.decrypt(nonce, ciphertext, None)
    elif algorithm == "chacha20-poly1305":
        cipher = ChaCha20Poly1305(symmetric_key)
        plaintext = cipher.decrypt(nonce, ciphertext, None)
    else:
        raise ValueError(f"Unsupported algorithm: {algorithm}")

    return plaintext
