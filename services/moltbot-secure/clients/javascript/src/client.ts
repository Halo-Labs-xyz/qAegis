/**
 * Main client for Moltbot Secure API
 */

import axios, { AxiosInstance } from "axios";
import { EncryptionKeyPair, encryptMessage, decryptMessage } from "./encryption";
import {
  MoltbotSecureError,
  AuthenticationError,
  SessionError,
  ProxyError,
} from "./exceptions";

export interface HealthResponse {
  status: string;
  service: string;
  version: string;
}

export interface LoginResponse {
  success: boolean;
  token: string;
  expires_at: number;
  user_id: string;
}

export interface SessionResponse {
  success: boolean;
  session_id: string;
  expires_at: number;
  encryption_enabled: boolean;
}

export interface KeyExchangeResponse {
  server_pubkey: number[];
  session_key: any;
  key_id: string;
  expires_at: number;
}

export interface MessageResponse {
  success: boolean;
  encrypted_response?: any;
  decrypted_response?: any;
  error?: string;
}

export class MoltbotSecureClient {
  private baseUrl: string;
  private axiosInstance: AxiosInstance;
  private token: string | null = null;
  private userId: string | null = null;
  private sessionId: string | null = null;
  private clientKeypair: EncryptionKeyPair | null = null;
  private serverKeypair: EncryptionKeyPair | null = null;

  constructor(baseUrl: string = "http://localhost:8443", verifySsl: boolean = true) {
    this.baseUrl = baseUrl.replace(/\/$/, "");
    this.axiosInstance = axios.create({
      baseURL: this.baseUrl,
      validateStatus: () => true, // Handle all status codes
    });
  }

  /**
   * Check if the service is healthy
   */
  async healthCheck(): Promise<HealthResponse> {
    try {
      const response = await this.axiosInstance.get("/health");
      if (response.status !== 200) {
        throw new MoltbotSecureError(`Health check failed: ${response.statusText}`);
      }
      return response.data;
    } catch (error: any) {
      throw new MoltbotSecureError(`Health check failed: ${error.message}`);
    }
  }

  /**
   * Register a new user
   */
  async register(username: string, password: string): Promise<any> {
    try {
      const response = await this.axiosInstance.post("/api/v1/auth/register", {
        username,
        password,
      });

      if (response.status !== 200) {
        throw new AuthenticationError(
          `Registration failed: ${response.data.error || response.statusText}`
        );
      }

      return response.data;
    } catch (error: any) {
      if (error instanceof AuthenticationError) {
        throw error;
      }
      throw new AuthenticationError(`Registration failed: ${error.message}`);
    }
  }

  /**
   * Login and get authentication token
   */
  async login(username: string, password: string): Promise<LoginResponse> {
    try {
      const response = await this.axiosInstance.post("/api/v1/auth/login", {
        username,
        password,
      });

      if (response.status !== 200) {
        throw new AuthenticationError(
          `Login failed: ${response.data.error || response.statusText}`
        );
      }

      const result = response.data;
      if (result.success) {
        this.token = result.token;
        this.userId = result.user_id;
        this.axiosInstance.defaults.headers.common[
          "Authorization"
        ] = `Bearer ${this.token}`;
      }

      return result;
    } catch (error: any) {
      if (error instanceof AuthenticationError) {
        throw error;
      }
      throw new AuthenticationError(`Login failed: ${error.message}`);
    }
  }

  /**
   * Create a new encrypted session
   */
  async createSession(): Promise<SessionResponse> {
    if (!this.token || !this.userId) {
      throw new AuthenticationError("Must login before creating session");
    }

    try {
      const response = await this.axiosInstance.post("/api/v1/session/create", {
        user_id: this.userId,
        token: this.token,
      });

      if (response.status !== 200) {
        throw new SessionError(
          `Session creation failed: ${response.data.error || response.statusText}`
        );
      }

      const result = response.data;
      if (result.success) {
        this.sessionId = result.session_id;
      }

      return result;
    } catch (error: any) {
      if (error instanceof SessionError) {
        throw error;
      }
      throw new SessionError(`Session creation failed: ${error.message}`);
    }
  }

  /**
   * Exchange encryption keys with the server
   */
  async exchangeKeys(): Promise<KeyExchangeResponse> {
    if (!this.sessionId) {
      throw new SessionError("Must create session before key exchange");
    }

    // Generate client keypair if not exists
    if (!this.clientKeypair) {
      this.clientKeypair = EncryptionKeyPair.generate();
    }

    try {
      const response = await this.axiosInstance.post("/api/v1/keys/exchange", {
        client_id: this.userId || "client",
        client_pubkey: Array.from(this.clientKeypair.publicKeyBytes()),
        session_id: this.sessionId,
      });

      if (response.status !== 200) {
        throw new MoltbotSecureError(
          `Key exchange failed: ${response.data.error || response.statusText}`
        );
      }

      return response.data;
    } catch (error: any) {
      throw new MoltbotSecureError(`Key exchange failed: ${error.message}`);
    }
  }

  /**
   * Send an encrypted message to Moltbot
   */
  async sendMessage(
    message: string,
    messageType: string = "text"
  ): Promise<MessageResponse> {
    if (!this.sessionId) {
      throw new SessionError("Must create session before sending messages");
    }
    if (!this.clientKeypair) {
      throw new SessionError("Must exchange keys before sending messages");
    }

    // Encrypt the message
    const messageBytes = Buffer.from(message, "utf-8");
    const encryptedMsg = encryptMessage(
      messageBytes,
      this.serverKeypair || this.clientKeypair,
      this.clientKeypair,
      "aes-256-gcm"
    );

    try {
      const response = await this.axiosInstance.post("/api/v1/proxy/message", {
        session_id: this.sessionId,
        encrypted_message: encryptedMsg,
        message_type: messageType,
      });

      if (response.status !== 200) {
        throw new ProxyError(
          `Failed to send message: ${response.data.error || response.statusText}`
        );
      }

      const result = response.data;

      // Decrypt the response if present
      if (result.success && result.encrypted_response) {
        const decrypted = decryptMessage(
          result.encrypted_response,
          this.clientKeypair!,
          null
        );
        result.decrypted_response = JSON.parse(decrypted.toString("utf-8"));
      }

      return result;
    } catch (error: any) {
      if (error instanceof ProxyError) {
        throw error;
      }
      throw new ProxyError(`Failed to send message: ${error.message}`);
    }
  }

  /**
   * Close the client session
   */
  close(): void {
    delete this.axiosInstance.defaults.headers.common["Authorization"];
    this.token = null;
    this.userId = null;
    this.sessionId = null;
  }
}
