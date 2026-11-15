# Pixel Village Production Deployment Guide

Complete guide for deploying Pixel Village game server to production.

## Quick Links

- **Repository**: https://github.com/gertyn230/game
- **Server**: Hetzner CAX11 (91.98.29.81)
- **Domain**: pixel-village.online
- **Admin Panel**: https://admin.pixel-village.online/admin

## Deployment Overview

```
Stage 1: Repository Preparation [✓ COMPLETE]
  ↓
Stage 2: Server Configuration [→ START HERE]
  ↓
Stage 3: Application Deployment
  ↓
Stage 4: Cloudflare Configuration
  ↓
Stage 5: Client Configuration
  ↓
Stage 6: Website Deployment
  ↓
Stage 7: Monitoring & Backups
  ↓
Stage 8: Final Verification & Launch
```

## Stage 1: Repository Preparation ✓

**Status**: COMPLETE

Changes made:
- ✓ Restored admin.html
- ✓ Removed junk files
- ✓ Fixed DATABASE_URL (persistent file storage)
- ✓ Enhanced admin security (header-based auth, POST for mutations)
- ✓ Improved Dockerfile (non-root user, healthcheck, ARM64)
- ✓ Created deployment configuration (docker-compose, Caddyfile)
- ✓ Updated admin.html with real API integration

## Stage 2: Server Configuration

**Time**: ~30 minutes

### Prerequisites
- Root or sudo access to server: 91.98.29.81
- SSH access configured

### Steps

1. **SSH to server as root**:
   ```bash
   ssh root@91.98.29.81
   ```

2. **Download and run setup script**:
   ```bash
   curl -fsSL https://raw.githubusercontent.com/gertyn230/game/main/deploy/setup-server.sh -o setup-server.sh
   chmod +x setup-server.sh
   sudo ./setup-server.sh
   ```

   This script will:
   - Update system packages
   - Enable automatic security updates
   - Install Docker and Docker Compose
   - Create 'deploy' user
   - Configure SSH security (key-only, no root)
   - Setup UFW firewall (ports 22, 80, 443)
   - Install and configure fail2ban

3. **Add SSH key for deploy user**:
   ```bash
   curl -fsSL https://raw.githubusercontent.com/gertyn230/game/main/deploy/add-ssh-key.sh -o add-ssh-key.sh
   chmod +x add-ssh-key.sh
   sudo ./add-ssh-key.sh
   ```

4. **Test SSH login as deploy user** (DO THIS BEFORE LOGGING OUT!):
   ```bash
   # From another terminal
   ssh deploy@91.98.29.81
   ```

5. **If SSH works, proceed. If not, fix before logging out as root!**

📖 **Detailed documentation**: `deploy/README.md`

## Stage 3: Application Deployment

**Time**: ~20 minutes

### Steps

1. **SSH as deploy user**:
   ```bash
   ssh deploy@91.98.29.81
   ```

2. **Run deployment script**:
   ```bash
   curl -fsSL https://raw.githubusercontent.com/gertyn230/game/main/deploy/deploy-app.sh -o deploy-app.sh
   chmod +x deploy-app.sh
   ./deploy-app.sh
   ```

   This will:
   - Clone repository
   - Setup environment (.env file)
   - Build Docker images
   - Start services (game server + Caddy)

3. **Configure ADMIN_KEY** (during script execution):
   - Script will generate secure key automatically
   - OR you can generate manually: `openssl rand -hex 32`
   - **SAVE THIS KEY!** You'll need it for admin panel

4. **Verify deployment**:
   ```bash
   docker compose ps
   curl http://localhost:8080/api/version
   ```

📖 **Detailed documentation**: `deploy/README.md`

## Stage 4: Cloudflare Configuration

**Time**: ~15 minutes

### Steps

1. Go to Cloudflare Dashboard → pixel-village.online → DNS

2. Add A records (all Proxied/Orange cloud):
   - `api.pixel-village.online` → `91.98.29.81`
   - `admin.pixel-village.online` → `91.98.29.81`
   - `pixel-village.online` → `91.98.29.81`
   - `www.pixel-village.online` → `91.98.29.81`

3. SSL/TLS Settings:
   - Go to SSL/TLS → Overview
   - Set mode: **Full (strict)**
   - Enable "Always Use HTTPS"

4. Security (optional but recommended):
   - Enable Bot Fight Mode
   - Add rate limiting for /api/admin/*
   - Configure Cloudflare Access for admin panel

5. Verify:
   ```bash
   curl https://api.pixel-village.online/api/version
   ```

📖 **Detailed documentation**: `deploy/STAGE4-CLOUDFLARE.md`

## Stage 5: Client Configuration

**Time**: ~10 minutes

### Steps

1. Go to GitHub repository → Settings → Secrets and variables → Actions

2. Update `PV_SERVER` variable:
   ```
   wss://api.pixel-village.online/ws
   ```

3. Trigger client rebuild:
   - Go to Actions tab
   - Run "Build" workflow
   - Wait for all platforms to build

4. Test:
   - Download built client
   - Launch game
   - Should connect to production server

📖 **Detailed documentation**: `deploy/STAGE5-CLIENT.md`

## Stage 6: Website Deployment

**Time**: ~20 minutes (if website exists)

### Option A: Cloudflare Pages (Recommended)
- Connect game-releases repository to Cloudflare Pages
- Configure custom domain: pixel-village.online

### Option B: Simple Coming Soon Page
- Already configured in Caddyfile
- Shows "Coming Soon" message

📖 **Detailed documentation**: `deploy/STAGE6-WEBSITE.md`

## Stage 7: Monitoring & Backups

**Time**: ~20 minutes

### Backups

1. **Setup automatic backups**:
   ```bash
   ssh deploy@91.98.29.81
   ~/pixel-village/deploy/setup-backup-cron.sh
   ```

2. **Test backup**:
   ```bash
   ~/pixel-village/deploy/backup-db.sh
   ls -lh ~/backups/
   ```

3. **Test restore**:
   ```bash
   ~/pixel-village/deploy/restore-db.sh
   ```

### Monitoring

1. **Setup UptimeRobot**:
   - Go to https://uptimerobot.com/
   - Add monitors for:
     - https://api.pixel-village.online/api/version
     - https://admin.pixel-village.online/admin
     - https://pixel-village.online/

2. **Configure email alerts**

📖 **Detailed documentation**: `deploy/STAGE7-MONITORING.md`

## Stage 8: Final Verification

**Time**: ~30 minutes

Complete the verification checklist to ensure everything works:

### Critical Tests

1. **Server endpoints**:
   ```bash
   curl https://api.pixel-village.online/api/version
   curl https://api.pixel-village.online/api/players
   ```

2. **WebSocket**:
   ```bash
   wscat -c wss://api.pixel-village.online/ws
   ```

3. **Admin panel**:
   - Open: https://admin.pixel-village.online/admin
   - Enter ADMIN_KEY
   - Test ban/unban functionality

4. **Game client**:
   - Launch game
   - Connect to server
   - Play test

5. **Backups**:
   - Verify backup ran successfully
   - Test restore process

📖 **Complete checklist**: `deploy/STAGE8-VERIFICATION.md`

## Quick Reference

### Useful Commands

```bash
# SSH to server
ssh deploy@91.98.29.81

# View logs
docker compose logs -f game-server
docker compose logs -f caddy

# Restart services
docker compose restart

# Backup database
~/pixel-village/deploy/backup-db.sh

# Update application
cd ~/pixel-village
git pull
cd deploy
docker compose up -d --build

# Check status
docker compose ps
docker compose logs --tail=50
```

### Important Files

- **Environment config**: `~/pixel-village/deploy/.env`
- **Database**: `/var/lib/pixel-village/data/game.db`
- **Backups**: `~/backups/`
- **Logs**: `docker compose logs`

### Emergency Contacts

- **Server**: Hetzner Cloud Console
- **Domain**: Cloudflare Dashboard  
- **Email**: gertn230@gmail.com

## Architecture Diagram

```
                    Internet
                       ↓
              ┌────────────────┐
              │   Cloudflare   │
              │  (DNS, SSL,    │
              │   DDoS, WAF)   │
              └────────┬───────┘
                       ↓
            ┌──────────────────────┐
            │  Hetzner Server      │
            │  91.98.29.81         │
            │                      │
            │  ┌────────────────┐  │
            │  │     Caddy      │  │
            │  │  (Proxy+TLS)   │  │
            │  └───────┬────────┘  │
            │          ↓            │
            │  ┌────────────────┐  │
            │  │  Game Server   │  │
            │  │  (Rust/Axum)   │  │
            │  └───────┬────────┘  │
            │          ↓            │
            │  ┌────────────────┐  │
            │  │    SQLite DB   │  │
            │  │   /data/game   │  │
            │  └────────────────┘  │
            └──────────────────────┘
```

## Security Checklist

- ✓ SSH key-only authentication
- ✓ Firewall (UFW + Cloudflare)
- ✓ fail2ban for SSH protection
- ✓ HTTPS only (forced)
- ✓ Admin key in headers (not URL)
- ✓ Rate limiting on admin endpoints
- ✓ Non-root Docker containers
- ✓ Automatic security updates
- ✓ Database backups

## Support

If you encounter issues:

1. Check logs: `docker compose logs`
2. Review documentation in `deploy/` folder
3. Verify checklist in `STAGE8-VERIFICATION.md`
4. Check UptimeRobot for downtime
5. Review Cloudflare security events

## Success Metrics

Your deployment is successful when:

- ✅ All services running (`docker compose ps`)
- ✅ API responds: https://api.pixel-village.online/api/version
- ✅ WebSocket connects: wss://api.pixel-village.online/ws
- ✅ Admin panel works: https://admin.pixel-village.online/admin
- ✅ Game clients can connect and play
- ✅ Backups running automatically
- ✅ Monitoring shows 100% uptime
- ✅ No errors in logs

## Timeline

Total estimated time: **~2.5 hours**

- Stage 2 (Server): 30 min
- Stage 3 (Deploy): 20 min
- Stage 4 (Cloudflare): 15 min
- Stage 5 (Client): 10 min
- Stage 6 (Website): 20 min
- Stage 7 (Monitoring): 20 min
- Stage 8 (Verification): 30 min

## Next Steps After Launch

1. Monitor for first 24 hours
2. Set up analytics (Google Analytics, Plausible)
3. Create status page
4. Configure off-site backups
5. Plan scaling strategy
6. Document operational procedures

---

**Ready to deploy?** Start with Stage 2! 🚀

**Questions?** Review the detailed documentation in each STAGE*.md file.
