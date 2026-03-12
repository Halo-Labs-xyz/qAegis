# Deployment Guide - Moltbot Secure

Complete guide for deploying Moltbot Secure in production.

## Prerequisites

- Docker and Docker Compose installed
- SSL certificates (Let's Encrypt recommended for production)
- Domain name (optional but recommended)
- Basic knowledge of Linux server administration

## Quick Start

### 1. Clone and Setup

```bash
git clone <repository-url>
cd services/moltbot-secure
```

### 2. Configure

```bash
# Copy example configuration
cp moltbot-secure.toml.example moltbot-secure.toml

# Edit configuration
nano moltbot-secure.toml
```

Update the following in `moltbot-secure.toml`:
- `bind_address`: Set to `0.0.0.0` for production
- `moltbot_url`: Your Moltbot instance URL
- `moltbot_api_key`: If required by your Moltbot instance
- `jwt_secret`: Generate a strong random secret (use `openssl rand -hex 32`)

### 3. Generate SSL Certificates

For development/testing:
```bash
./nginx/generate-ssl.sh
```

For production (Let's Encrypt):
```bash
# Install certbot
sudo apt-get install certbot

# Generate certificates
sudo certbot certonly --standalone -d your-domain.com

# Copy certificates to nginx/ssl/
sudo cp /etc/letsencrypt/live/your-domain.com/fullchain.pem nginx/ssl/cert.pem
sudo cp /etc/letsencrypt/live/your-domain.com/privkey.pem nginx/ssl/key.pem
sudo chmod 600 nginx/ssl/key.pem
```

### 4. Deploy

```bash
# Run deployment script
./scripts/deploy.sh

# Or manually:
docker-compose up -d
```

### 5. Verify

```bash
# Check health
curl https://your-domain.com/health

# Check logs
docker-compose logs -f
```

## Production Deployment

### Using Docker Compose

1. **Update docker-compose.yml**:
   - Set proper resource limits
   - Configure volumes for persistent storage
   - Set up proper networking

2. **Environment Variables**:
   ```bash
   # Create .env file
   cat > .env << EOF
   MOLTBOT_URL=http://moltbot:3000
   JWT_SECRET=$(openssl rand -hex 32)
   RUST_LOG=info
   EOF
   ```

3. **Deploy**:
   ```bash
   docker-compose -f docker-compose.yml up -d
   ```

### Using Kubernetes

See `k8s/` directory for Kubernetes manifests (create if needed).

### Manual Deployment

1. **Build**:
   ```bash
   cargo build --release
   ```

2. **Install**:
   ```bash
   sudo cp target/release/moltbot-secure /usr/local/bin/
   sudo mkdir -p /etc/moltbot-secure
   sudo cp moltbot-secure.toml /etc/moltbot-secure/
   ```

3. **Create systemd service**:
   ```bash
   sudo tee /etc/systemd/system/moltbot-secure.service << EOF
   [Unit]
   Description=Moltbot Secure Service
   After=network.target

   [Service]
   Type=simple
   User=moltbot
   WorkingDirectory=/var/lib/moltbot-secure
   ExecStart=/usr/local/bin/moltbot-secure
   Restart=always
   RestartSec=10

   [Install]
   WantedBy=multi-user.target
   EOF

   sudo systemctl enable moltbot-secure
   sudo systemctl start moltbot-secure
   ```

## Nginx Configuration

The provided `nginx/nginx.conf` includes:
- HTTPS with modern TLS configuration
- Rate limiting
- Security headers
- Health check endpoint
- Proper proxy settings

### Customization

Edit `nginx/nginx.conf` to:
- Update server_name
- Adjust rate limits
- Add IP whitelisting
- Configure logging

## Monitoring

### Health Checks

```bash
# Service health
curl http://localhost:8443/health

# Through nginx
curl https://your-domain.com/health
```

### Logs

```bash
# Docker logs
docker-compose logs -f moltbot-secure

# Nginx logs
docker-compose logs -f nginx

# System logs (if using systemd)
journalctl -u moltbot-secure -f
```

### Metrics

Consider integrating:
- Prometheus metrics endpoint
- Grafana dashboards
- Alerting rules

## Security Checklist

- [ ] Strong JWT secret configured
- [ ] SSL certificates valid and up-to-date
- [ ] Firewall rules configured
- [ ] Rate limiting enabled
- [ ] Security headers configured
- [ ] Regular security updates
- [ ] Key rotation enabled
- [ ] Backup strategy in place
- [ ] Monitoring and alerting set up
- [ ] Access logs reviewed regularly

## Troubleshooting

### Service won't start

```bash
# Check logs
docker-compose logs moltbot-secure

# Check configuration
docker-compose config

# Verify ports
netstat -tulpn | grep 8443
```

### SSL certificate issues

```bash
# Verify certificate
openssl x509 -in nginx/ssl/cert.pem -text -noout

# Check certificate expiration
openssl x509 -in nginx/ssl/cert.pem -noout -dates
```

### Connection errors

```bash
# Test connectivity
curl -v https://your-domain.com/health

# Check nginx configuration
docker-compose exec nginx nginx -t
```

## Backup and Recovery

### Backup

```bash
# Backup configuration
tar -czf moltbot-secure-backup-$(date +%Y%m%d).tar.gz \
  moltbot-secure.toml \
  keys/ \
  nginx/ssl/
```

### Recovery

```bash
# Restore from backup
tar -xzf moltbot-secure-backup-YYYYMMDD.tar.gz
docker-compose restart
```

## Updates

```bash
# Pull latest changes
git pull

# Rebuild and restart
docker-compose build
docker-compose up -d
```

## Support

For issues and questions:
- Check logs first
- Review configuration
- Consult documentation
- Open an issue on GitHub
