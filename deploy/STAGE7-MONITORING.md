# Stage 7: Monitoring and Backups

## Automated Backups

Database backups are critical for data safety.

### Setup Automatic Daily Backups

```bash
# SSH to server
ssh deploy@91.98.29.81

# Make backup scripts executable
chmod +x ~/pixel-village/deploy/backup-db.sh
chmod +x ~/pixel-village/deploy/restore-db.sh
chmod +x ~/pixel-village/deploy/setup-backup-cron.sh

# Setup automatic daily backups at 3 AM
~/pixel-village/deploy/setup-backup-cron.sh
```

### Manual Backup

```bash
~/pixel-village/deploy/backup-db.sh
```

### Restore from Backup

```bash
~/pixel-village/deploy/restore-db.sh
# Follow interactive prompts
```

### Backup Storage

Backups are stored in: `/home/deploy/backups/`

**Retention policy**: 14 days (configurable in `backup-db.sh`)

**Backup format**: `game-backup-YYYYMMDD-HHMMSS.db.gz`

### Test Backup Recovery

It's important to test that backups actually work:

```bash
# 1. Create a test backup
~/pixel-village/deploy/backup-db.sh

# 2. Make a small change (add test data)
docker exec pixel-village-server sqlite3 /data/game.db "INSERT INTO admin_logs (admin, action, timestamp) VALUES ('test', 'backup test', $(date +%s))"

# 3. Restore from backup
~/pixel-village/deploy/restore-db.sh
# Choose the backup you just made

# 4. Verify the test data is gone (backup was restored)
docker exec pixel-village-server sqlite3 /data/game.db "SELECT * FROM admin_logs ORDER BY id DESC LIMIT 5"
```

### Off-Server Backups (Recommended)

For maximum safety, also backup to external location:

#### Option A: Hetzner Storage Box

Hetzner offers backup storage boxes.

1. Order Storage Box from Hetzner
2. Setup rsync:
   ```bash
   # Add to backup-db.sh after creating backup:
   rsync -avz /home/deploy/backups/ user@your-storage-box.de:backup/pixel-village/
   ```

#### Option B: AWS S3

```bash
# Install AWS CLI
sudo apt install awscli

# Configure
aws configure

# Add to backup-db.sh:
aws s3 sync /home/deploy/backups/ s3://your-bucket/pixel-village-backups/
```

#### Option C: Manual Download

Regularly download backups to your local machine:

```bash
# On your local machine
scp deploy@91.98.29.81:~/backups/game-backup-*.db.gz ~/local-backups/
```

## Uptime Monitoring

Monitor that your server is accessible 24/7.

### UptimeRobot (Free, Recommended)

1. Go to https://uptimerobot.com/
2. Sign up (free account monitors up to 50 sites)
3. Add monitors:

   **Monitor 1: API Server**
   - Monitor Type: HTTP(s)
   - Friendly Name: Pixel Village API
   - URL: `https://api.pixel-village.online/api/version`
   - Monitoring Interval: 5 minutes
   - Alert Contacts: Your email

   **Monitor 2: Admin Panel**
   - Monitor Type: HTTP(s)
   - Friendly Name: Pixel Village Admin
   - URL: `https://admin.pixel-village.online/admin`
   - Monitoring Interval: 5 minutes
   - Alert Contacts: Your email

   **Monitor 3: Website**
   - Monitor Type: HTTP(s)
   - Friendly Name: Pixel Village Website
   - URL: `https://pixel-village.online/`
   - Monitoring Interval: 5 minutes
   - Alert Contacts: Your email

4. Configure alert settings:
   - Enable email notifications
   - Set alert threshold: "Down for 2 minutes"

### Alternative: BetterUptime

Free plan with status page:
1. Go to https://betteruptime.com/
2. Create monitors similar to UptimeRobot
3. Create public status page: `status.pixel-village.online`

## Log Viewing

### Docker Logs

```bash
# View all logs
docker compose logs

# Follow logs in real-time
docker compose logs -f

# Game server logs
docker compose logs -f game-server

# Caddy logs
docker compose logs -f caddy

# Last 100 lines
docker compose logs --tail=100 game-server
```

### Caddy Access Logs

Caddy logs are configured in Caddyfile to write to files:

```bash
# View API access logs
docker compose exec caddy cat /var/log/caddy/api.log

# View admin access logs
docker compose exec caddy cat /var/log/caddy/admin.log

# View website access logs
docker compose exec caddy cat /var/log/caddy/www.log

# Follow logs in real-time
docker compose exec caddy tail -f /var/log/caddy/api.log
```

### Database Admin Logs

View admin actions logged in database:

```bash
# Last 20 admin actions
docker exec pixel-village-server sqlite3 /data/game.db \
  "SELECT datetime(timestamp, 'unixepoch') as time, admin, action FROM admin_logs ORDER BY id DESC LIMIT 20"

# Actions from last 24 hours
docker exec pixel-village-server sqlite3 /data/game.db \
  "SELECT datetime(timestamp, 'unixepoch') as time, admin, action FROM admin_logs WHERE timestamp > $(date -d '24 hours ago' +%s) ORDER BY id DESC"
```

## System Monitoring

### Basic Server Stats

```bash
# Disk usage
df -h

# Memory usage
free -h

# CPU usage
top

# Docker container stats
docker stats

# Check Docker volumes
docker volume ls
docker system df
```

### Disk Space Monitoring

Setup alert when disk space is low:

```bash
# Create monitoring script
cat > ~/check-disk.sh <<'EOF'
#!/bin/bash
THRESHOLD=80
CURRENT=$(df / | tail -1 | awk '{print $5}' | sed 's/%//')
if [ $CURRENT -gt $THRESHOLD ]; then
    echo "WARNING: Disk usage is ${CURRENT}% (threshold: ${THRESHOLD}%)"
    # Add email notification here if needed
fi
EOF

chmod +x ~/check-disk.sh

# Add to crontab (check daily at noon)
(crontab -l 2>/dev/null; echo "0 12 * * * /home/deploy/check-disk.sh") | crontab -
```

### Clean Up Docker

If disk space is running low:

```bash
# Remove unused images, containers, networks
docker system prune -a

# Remove old logs
docker compose logs --tail=0 game-server
```

## Performance Monitoring

### Database Size

```bash
# Check database size
docker exec pixel-village-server ls -lh /data/game.db

# Check database integrity
docker exec pixel-village-server sqlite3 /data/game.db "PRAGMA integrity_check"

# Vacuum database (optimize)
docker exec pixel-village-server sqlite3 /data/game.db "VACUUM"
```

### Connection Stats

```bash
# Check current connections
docker exec pixel-village-server netstat -an | grep :8080 | wc -l

# Check if server is responding
curl -I https://api.pixel-village.online/api/version
```

## Hetzner Cloud Backups

Enable automated snapshots in Hetzner Cloud:

1. Go to Hetzner Cloud Console
2. Select your server (pixel-village)
3. Click "Backups"
4. Enable "Backups"
   - Cost: Small fee per month
   - Creates weekly automated snapshots
   - Kept for 7 days
5. Can manually create snapshots anytime

**When to use server snapshots:**
- Before major updates
- Before system changes
- As disaster recovery backup

## Monitoring Checklist

Create a weekly/monthly checklist:

### Weekly:
- [ ] Check UptimeRobot status (no downtime?)
- [ ] Verify backups are running (`ls -lh ~/backups/`)
- [ ] Check disk space (`df -h`)
- [ ] Review error logs (`docker compose logs --tail=50`)

### Monthly:
- [ ] Test backup restoration
- [ ] Review admin logs for suspicious activity
- [ ] Update system packages (`sudo apt update && sudo apt upgrade`)
- [ ] Check database size growth
- [ ] Review Cloudflare analytics
- [ ] Vacuum database if needed

### Quarterly:
- [ ] Security audit (check UFW rules, fail2ban logs)
- [ ] Performance review (response times, server load)
- [ ] Update Docker images
- [ ] Review and update documentation

## Emergency Procedures

### Server is Down

1. Check UptimeRobot alerts
2. SSH to server and check status:
   ```bash
   docker compose ps
   docker compose logs --tail=50
   ```
3. Restart if needed:
   ```bash
   docker compose restart
   ```

### Database Corrupted

1. Stop server
2. Restore from backup:
   ```bash
   ~/pixel-village/deploy/restore-db.sh
   ```
3. Start server

### High Server Load

1. Check what's using resources:
   ```bash
   docker stats
   top
   ```
2. Check for DDoS (unusual traffic patterns)
3. Review Cloudflare analytics
4. May need to scale server (upgrade Hetzner plan)

### Disk Full

1. Check what's using space:
   ```bash
   du -sh /* | sort -h
   ```
2. Clean up Docker:
   ```bash
   docker system prune -a
   ```
3. Remove old backups manually if needed
4. Consider upgrading server storage

## Support Contacts

Keep these handy:

- **Hetzner Support**: https://console.hetzner.cloud/
- **Cloudflare Support**: https://dash.cloudflare.com/
- **GitHub Support**: https://support.github.com/
- **Your Email**: gertn230@gmail.com

## Next: Stage 8 Verification

After monitoring is set up, proceed to Stage 8 final verification checklist.
