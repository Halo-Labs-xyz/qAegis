/**
 * Custom exceptions for Moltbot Secure client
 */

export class MoltbotSecureError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "MoltbotSecureError";
    Object.setPrototypeOf(this, MoltbotSecureError.prototype);
  }
}

export class AuthenticationError extends MoltbotSecureError {
  constructor(message: string) {
    super(message);
    this.name = "AuthenticationError";
    Object.setPrototypeOf(this, AuthenticationError.prototype);
  }
}

export class EncryptionError extends MoltbotSecureError {
  constructor(message: string) {
    super(message);
    this.name = "EncryptionError";
    Object.setPrototypeOf(this, EncryptionError.prototype);
  }
}

export class SessionError extends MoltbotSecureError {
  constructor(message: string) {
    super(message);
    this.name = "SessionError";
    Object.setPrototypeOf(this, SessionError.prototype);
  }
}

export class ProxyError extends MoltbotSecureError {
  constructor(message: string) {
    super(message);
    this.name = "ProxyError";
    Object.setPrototypeOf(this, ProxyError.prototype);
  }
}
