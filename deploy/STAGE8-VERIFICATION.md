# Stage 8: Final Verification Checklist

Complete system verification before going live.

## Pre-Flight Checklist

### ✅ Server Configuration

- [ ] Server is running Ubuntu 24.04 ARM64
- [ ] System packages are up to date
- [ ] Automatic security updates enabled (unattended-upgrades)
- [ ] Docker and Docker Compose installed
- [ ] Non-root user 'deploy' created and configured
- [ ] User 'deploy' added to docker group

### ✅ SSH & Security

- [ ] SSH key-only authentication enabled
- [ ] Password authentication disabled
- [ ] Root login disabled
- [ ] SSH works with deploy user: `ssh deploy@91.98.29.81`
- [ ] fail2ban installed and running
- [ ] UFW firewall enabled
- [ ] Ports 22, 80, 443 open; 8080 closed to external

### ✅ Application Deployment

- [ ] Repository cloned to `/home/deploy/pixel-village`
- [ ] `.env` file created with secure ADMIN_KEY
- [ ] Docker images built successfully
- [ ] Docker containers running: `docker compose ps`
- [ ] Game server container healthy
- [ ] Caddy container running
- [ ] Database file exists at `/var/lib/pixel-village/data/game.db` or in volume
- [ ] Data volume mounted correctly

### ✅ DNS & Domain

- [ ] A record: `api.pixel-village.online` → `91.98.29.81` (Proxied)
- [ ] A record: `admin.pixel-village.online` → `91.98.29.81` (Proxied)
- [ ] A record: `pixel-village.online` → `91.98.29.81` (Proxied)
- [ ] A record: `www.pixel-village.online` → `91.98.29.81` (Proxied)
- [ ] DNS propagation complete (check: `nslookup api.pixel-village.online`)

### ✅ SSL/TLS

- [ ] Cloudflare SSL mode: Full (strict)
- [ ] Caddy has obtained SSL certificates (check logs)
- [ ] HTTPS works: `curl https://api.pixel-village.online/api/version`
- [ ] Always Use HTTPS enabled in Cloudflare
- [ ] No SSL certificate errors

### ✅ API Endpoints

Test all endpoints:

```bash
# Public endpoints (no auth required)
curl https://api.pixel-village.online/
curl https://api.pixel-village.online/api/version
curl https://api.pixel-village.online/api/players

# Should return data without auth errors
```

- [ ] Root endpoint (/) returns online players list
- [ ] /api/version returns version info
- [ ] /api/players returns player list
- [ ] WebSocket endpoint accessible (test from client)

### ✅ Admin Panel

```bash
# Test admin panel access
curl -I https://admin.pixel-village.online/admin

# Should return 200 OK
```

- [ ] Admin panel loads: https://admin.pixel-village.online/admin
- [ ] Prompted for ADMIN_KEY on first load
- [ ] Can authenticate with correct ADMIN_KEY
- [ ] Invalid key shows authorization error
- [ ] Rate limiting works (5 failed attempts locks out)

Test admin API with correct ADMIN_KEY:

```bash
ADMIN_KEY="your-admin-key-here"

# Test stats endpoint (GET with header)
curl -H "X-Admin-Key: $ADMIN_KEY" https://api.pixel-village.online/api/admin/stats

# Test ban endpoint (POST with JSON)
curl -X POST -H "X-Admin-Key: $ADMIN_KEY" -H "Content-Type: application/json" \
  -d '{"player":"TestPlayer","reason":"Test ban","hours":1}' \
  https://api.pixel-village.online/api/admin/ban

# Should return success response
```

- [ ] Admin stats endpoint works
- [ ] Ban/unban works (POST with JSON body)
- [ ] Mute/unmute works (POST with JSON body)
- [ ] Bans list loads
- [ ] Admin logs show actions
- [ ] Unauthorized without key (401)
- [ ] Rate limited after 5 failed attempts (429)

### ✅ WebSocket

Test WebSocket connection:

```bash
# Install wscat if needed
npm install -g wscat

# Test WebSocket
wscat -c wss://api.pixel-village.online/ws

# Should connect successfully
```

- [ ] WebSocket connects: `wss://api.pixel-village.online/ws`
- [ ] Game client can connect
- [ ] No proxy/timeout issues
- [ ] Cloudflare WebSocket enabled

### ✅ Database

```bash
# Check database
docker exec pixel-village-server sqlite3 /data/game.db "PRAGMA integrity_check"
docker exec pixel-village-server sqlite3 /data/game.db ".tables"
docker exec pixel-village-server ls -lh /data/game.db
```

- [ ] Database file exists and is writable
- [ ] Tables created (admin_logs, server_config, tickets, etc.)
- [ ] Database integrity OK
- [ ] Data persists after container restart
- [ ] Database NOT in-memory (sqlite::memory:)

### ✅ Security

- [ ] Server accessible only through Cloudflare
- [ ] Direct IP access blocked for 8080 (check firewall)
- [ ] ADMIN_KEY is strong (32+ characters)
- [ ] ADMIN_KEY not committed to git
- [ ] No secrets in logs
- [ ] Cloudflare WAF enabled
- [ ] Bot protection enabled
- [ ] DDoS protection active

Test security:

```bash
# Should be blocked by firewall
curl http://91.98.29.81:8080/api/version

# Should work through Cloudflare
curl https://api.pixel-village.online/api/version
```

### ✅ Backups

- [ ] Backup script exists and is executable
- [ ] Cron job configured for daily backups
- [ ] Manual backup tested: `~/pixel-village/deploy/backup-db.sh`
- [ ] Backup file created in `/home/deploy/backups/`
- [ ] Restore tested successfully
- [ ] Backups retained for 14 days

Test backup and restore:

```bash
# Create backup
~/pixel-village/deploy/backup-db.sh

# List backups
ls -lh ~/backups/

# Test restore (use safety backup!)
~/pixel-village/deploy/restore-db.sh
```

### ✅ Monitoring

- [ ] UptimeRobot monitors configured
  - [ ] API monitor: https://api.pixel-village.online/api/version
  - [ ] Admin monitor: https://admin.pixel-village.online/admin
  - [ ] Website monitor: https://pixel-village.online/
- [ ] Email alerts configured
- [ ] Docker health checks working
- [ ] Logs accessible and rotated

Check monitoring:

```bash
# Check Docker health
docker compose ps

# View logs
docker compose logs --tail=50 game-server

# Check disk space
df -h
```

### ✅ Client Configuration

- [ ] GitHub Actions variable `PV_SERVER` set to correct URL
- [ ] Clients rebuilt with production server address
- [ ] Game client can connect to server
- [ ] Auto-update checks `/api/version`
- [ ] Download link points to GitHub Releases

### ✅ Website

- [ ] Main website accessible: https://pixel-village.online/
- [ ] WWW redirect works: https://www.pixel-village.online/
- [ ] SSL certificate valid
- [ ] Mobile responsive

### ✅ Performance

Test under load:

```bash
# Simple load test
for i in {1..10}; do
  curl -s https://api.pixel-village.online/api/version &
done
wait
```

- [ ] Server handles concurrent requests
- [ ] Response time < 500ms
- [ ] No memory leaks
- [ ] CPU usage reasonable
- [ ] Database queries optimized

## Smoke Test: Complete User Flow

Simulate a complete user journey:

1. **User visits website**
   - Go to https://pixel-village.online/
   - Should load without errors

2. **User downloads game**
   - Click download link
   - Should redirect to GitHub Releases

3. **User opens game client**
   - Launch game
   - Should show connection to server

4. **User connects to server**
   - Game connects to `wss://api.pixel-village.online/ws`
   - Should join game world

5. **Admin monitors player**
   - Open https://admin.pixel-village.online/admin
   - Enter ADMIN_KEY
   - Should see player in stats

6. **Admin takes action**
   - Test ban/unban player
   - Should reflect in game

## Disaster Recovery Test

Test that you can recover from failure:

1. **Backup current state**
   ```bash
   ~/pixel-village/deploy/backup-db.sh
   ```

2. **Simulate failure**
   ```bash
   docker compose stop
   docker compose down
   ```

3. **Recover**
   ```bash
   docker compose up -d
   # Or restore from backup if needed
   ```

4. **Verify everything works**
   - Game connects
   - Data intact
   - Admin works

## Final Security Audit

- [ ] Review UFW rules: `sudo ufw status`
- [ ] Check fail2ban: `sudo fail2ban-client status sshd`
- [ ] Review admin logs for suspicious activity
- [ ] Verify no sensitive data in logs
- [ ] Check for open ports: `sudo netstat -tulpn`
- [ ] Review Cloudflare security events

## Documentation Review

- [ ] README.md is up to date
- [ ] All stage documentation complete
- [ ] Emergency procedures documented
- [ ] ADMIN_KEY stored securely (password manager)
- [ ] Server access credentials secured

## Launch Day Checklist

Before announcing to users:

- [ ] All verification tests pass
- [ ] Monitoring shows all systems operational
- [ ] Backup confirmed working
- [ ] Game clients distributed with correct server
- [ ] Admin panel accessible
- [ ] Support channels ready
- [ ] Status page available (optional)

## Post-Launch Monitoring

First 24 hours after launch:

- [ ] Monitor UptimeRobot for any downtime
- [ ] Watch server resources (CPU, memory, disk)
- [ ] Check for errors in logs every 2-4 hours
- [ ] Monitor player count
- [ ] Test admin functions
- [ ] Verify backups run successfully

## Common Issues & Solutions

### Issue: SSL Certificate Error
**Solution**: Check Caddy logs, verify DNS is correct, wait for certificate

### Issue: WebSocket Won't Connect
**Solution**: Check Cloudflare WebSocket enabled, test with wscat

### Issue: Admin Panel 401 Unauthorized
**Solution**: Verify ADMIN_KEY in .env matches key being used

### Issue: Database Not Persisting
**Solution**: Check volume mount, verify DATABASE_URL not using :memory:

### Issue: Server Can't Be Reached
**Solution**: Check firewall (UFW), verify DNS, check Docker containers

### Issue: High Memory Usage
**Solution**: Check for memory leaks, restart containers, consider upgrade

## Success Criteria

System is ready for production when:

✅ All endpoints respond correctly
✅ WebSocket connections stable
✅ Admin panel functional and secure
✅ Backups running automatically
✅ Monitoring alerts configured
✅ SSL certificates valid
✅ No errors in logs
✅ Performance acceptable under load
✅ Disaster recovery tested

## 🎉 Launch!

Once all checks pass:

1. Announce to users
2. Share download links
3. Monitor closely for first 24 hours
4. Be ready to respond to issues
5. Celebrate successful deployment!

## Support Resources

- **Documentation**: ~/pixel-village/deploy/README.md
- **Logs**: `docker compose logs`
- **Database**: `docker exec pixel-village-server sqlite3 /data/game.db`
- **Backups**: ~/backups/
- **Monitoring**: UptimeRobot dashboard

**Remember**: You've built a complete production system with proper security, monitoring, and backups. You're ready! 🚀
