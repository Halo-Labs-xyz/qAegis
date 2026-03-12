#!/bin/bash
# Generate self-signed SSL certificate for development/testing
# For production, use certificates from a trusted CA (Let's Encrypt, etc.)

set -e

SSL_DIR="./nginx/ssl"
mkdir -p "$SSL_DIR"

echo "Generating self-signed SSL certificate..."
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
    -keyout "$SSL_DIR/key.pem" \
    -out "$SSL_DIR/cert.pem" \
    -subj "/C=US/ST=State/L=City/O=Organization/CN=localhost"

chmod 600 "$SSL_DIR/key.pem"
chmod 644 "$SSL_DIR/cert.pem"

echo "SSL certificate generated in $SSL_DIR/"
echo "For production, replace these with certificates from a trusted CA"
