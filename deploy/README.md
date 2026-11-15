# Pixel Village Server Deployment

This directory contains production deployment configuration for Pixel Village game server.

## Prerequisites

- Hetzner Cloud CAX11 (ARM64) server with Ubuntu 24.04 LTS
- Docker and Docker Compose installed
- Domain: pixel-village.online configured in Cloudflare DNS
- SSH access to the server

## Quick Start

### 1. Server Setup (STAGE 2)

Run on the server via SSH:

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER
newgrp docker

# Install Docker Compose plugin
sudo apt install docker-compose-plugin -y

# Verify installation
docker --version
docker compose version
```

### 2. Clone Repository

```bash
cd ~
git clone https://github.com/gertyn230/game.git pixel-village
cd pixel-village
```

### 3. Configure Environment

```bash
# Copy environment template
cp .env.example .env

# Generate secure admin key
ADMIN_KEY=$(openssl rand -hex 32)

# Edit .env file
nano .env
```

Set these values in `.env`:
```bash
PORT=8080
DATABASE_URL=sqlite:///data/game.db?mode=rwc
ADMIN_KEY=<your-generated-key-here>
```

**IMPORTANT:** Keep your `ADMIN_KEY` secret! Do not commit it to git.

### 4. Build and Run

```bash
cd deploy
docker compose up -d --build
```

### 5. Check Status

```bash
# View logs
docker compose logs -f game-server

# Check health
docker compose ps

# Test endpoints
curl http://localhost:8080/api/version
```

## DNS Configuration (STAGE 4)

Configure these A records in Cloudflare DNS (Proxied):

- `api.pixel-village.online` → `91.98.29.81`
- `admin.pixel-village.online` → `91.98.29.81`
- `pixel-village.online` → `91.98.29.81`
- `www.pixel-village.online` → `91.98.29.81`

## SSL/TLS Options

### Option 1: Let's Encrypt (Recommended for simplicity)

Caddy will automatically obtain and renew SSL certificates from Let's Encrypt. No additional configuration needed.

**Pros:**
- Fully automated
- No manual certificate management
- Free

**Cons:**
- Cloudflare must be in DNS-only mode (gray cloud) OR use DNS challenge

### Option 2: Cloudflare Origin Certificate (Recommended for security)

1. Go to Cloudflare Dashboard → SSL/TLS → Origin Server
2. Create Origin Certificate (15-year validity)
3. Save certificate and private key
4. Update Caddyfile to use them:

```caddyfile
api.pixel-village.online {
    tls /path/to/cert.pem /path/to/key.pem
    # ... rest of config
}
```

5. Set Cloudflare SSL mode to "Full (strict)"

**Pros:**
- Works with Proxied DNS (orange cloud)
- Better security with Cloudflare WAF
- Hides origin IP

**Cons:**
- Manual certificate renewal every 15 years

## Security Checklist

- [ ] Changed default ADMIN_KEY
- [ ] Configured UFW firewall (ports 22, 80, 443 only)
- [ ] SSH key-only authentication enabled
- [ ] fail2ban installed and configured
- [ ] Database backups configured
- [ ] Cloudflare Access configured for admin panel

## Firewall Setup

```bash
# Install UFW
sudo apt install ufw -y

# Default policies
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow SSH
sudo ufw allow 22/tcp

# Allow HTTP/HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Enable firewall
sudo ufw enable
sudo ufw status
```

## Backup Database

```bash
# Manual backup
docker exec pixel-village-server sqlite3 /data/game.db ".backup /data/backup-$(date +%Y%m%d-%H%M%S).db"

# Copy backup to host
docker cp pixel-village-server:/data/backup-*.db ~/backups/
```

## Troubleshooting

### Check logs
```bash
docker compose logs -f game-server
docker compose logs -f caddy
```

### Restart services
```bash
docker compose restart game-server
docker compose restart caddy
```

### Rebuild after code changes
```bash
git pull
docker compose up -d --build
```

### Database issues
```bash
# Check database file
docker exec pixel-village-server ls -lh /data/game.db

# Verify database integrity
docker exec pixel-village-server sqlite3 /data/game.db "PRAGMA integrity_check"
```

## Admin Panel Access

Admin panel: `https://admin.pixel-village.online/admin`

Authentication:
- Add `X-Admin-Key` header with your ADMIN_KEY value
- Or use `Authorization: Bearer <ADMIN_KEY>` header

## API Endpoints

### Public Endpoints
- `GET /` - Online players list
- `GET /api/version` - Game version info
- `GET /api/players` - Current players
- `WS /ws` - WebSocket connection for game client

### Admin Endpoints (require X-Admin-Key header)
- `GET /admin` - Admin panel HTML
- `POST /api/admin/ban` - Ban player
- `POST /api/admin/unban` - Unban player
- `POST /api/admin/mute` - Mute player
- `POST /api/admin/unmute` - Unmute player
- `GET /api/admin/bans` - List all bans
- `GET /api/admin/stats` - Server statistics
- `GET /api/admin/logs` - Admin action logs
- `GET /api/admin/tickets` - View tickets

## Architecture

```
Internet
    ↓
Cloudflare (DNS + DDoS protection)
    ↓
Hetzner Server (91.98.29.81)
    ↓
Caddy (Reverse Proxy + TLS)
    ↓
Docker Network
    ↓
Game Server (Rust + Axum + SQLite)
```

## Monitoring

- Uptime: Configure UptimeRobot to monitor `https://api.pixel-village.online/api/version`
- Logs: `docker compose logs`
- Metrics: Built-in Docker health checks

## Updates

```bash
# Pull latest code
git pull

# Rebuild and restart
docker compose up -d --build

# No downtime: old container keeps running until new one is healthy
```
