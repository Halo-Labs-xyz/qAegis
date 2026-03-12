"""
Custom exceptions for Moltbot Secure client
"""


class MoltbotSecureError(Exception):
    """Base exception for Moltbot Secure client errors"""
    pass


class AuthenticationError(MoltbotSecureError):
    """Authentication-related errors"""
    pass


class EncryptionError(MoltbotSecureError):
    """Encryption-related errors"""
    pass


class SessionError(MoltbotSecureError):
    """Session-related errors"""
    pass


class ProxyError(MoltbotSecureError):
    """Proxy communication errors"""
    pass
