# ============================================================================
# QRMS Docker Image
# ============================================================================

FROM rust:latest AS builder

WORKDIR /build

# Copy source
COPY Cargo.toml ./
COPY src ./src
COPY static ./static

# Build release (generates Cargo.lock)
RUN cargo build --release --bin qrms

# ============================================================================
# Runtime
# ============================================================================

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary and static files
COPY --from=builder /build/target/release/qrms /app/qrms
COPY --from=builder /build/static /app/static

# Create data directory
RUN mkdir -p /data

EXPOSE 5050
EXPOSE 8081

ENV RUST_LOG=info

CMD ["/app/qrms"]
