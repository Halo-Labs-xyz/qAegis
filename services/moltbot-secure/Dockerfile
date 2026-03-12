# Multi-stage build for Moltbot Secure
FROM rust:1.70-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/moltbot-secure /usr/local/bin/moltbot-secure

# Create directories for config and keys
RUN mkdir -p /app/config /app/keys && \
    chmod 700 /app/keys

# Copy example config
COPY moltbot-secure.toml.example /app/config/moltbot-secure.toml.example

# Set working directory and config path
WORKDIR /app
ENV MOLTBOT_SECURE_CONFIG=/app/config/moltbot-secure.toml

# Expose port
EXPOSE 8443

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8443/health || exit 1

# Run the application
CMD ["moltbot-secure"]
