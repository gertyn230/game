# Stage 4: Cloudflare Configuration

Follow these steps in Cloudflare Dashboard to configure DNS and security.

## DNS Configuration

1. Go to Cloudflare Dashboard → Your domain (pixel-village.online) → DNS → Records

2. Add the following A records:

   | Type | Name  | Content      | Proxy status | TTL  |
   |------|-------|--------------|--------------|------|
   | A    | api   | 91.98.29.81  | Proxied (🟠) | Auto |
   | A    | admin | 91.98.29.81  | Proxied (🟠) | Auto |
   | A    | @     | 91.98.29.81  | Proxied (🟠) | Auto |
   | A    | www   | 91.98.29.81  | Proxied (🟠) | Auto |

3. Click "Save" for each record

## SSL/TLS Configuration

1. Go to SSL/TLS → Overview
2. Select SSL/TLS encryption mode: **Full (strict)**
3. Click "Save"

### Option A: Let's Encrypt (Automatic)

Caddy will automatically obtain certificates. **Requires DNS records to be in DNS-only mode (gray cloud) during initial setup.**

Steps:
1. Temporarily switch DNS records to "DNS only" (gray cloud)
2. Wait for Caddy to obtain certificates (check logs)
3. Switch back to "Proxied" (orange cloud)

### Option B: Cloudflare Origin Certificate (Recommended)

1. Go to SSL/TLS → Origin Server
2. Click "Create Certificate"
3. Select:
   - Private key type: RSA (2048)
   - Hostnames: `*.pixel-village.online`, `pixel-village.online`
   - Certificate Validity: 15 years
4. Click "Create"
5. **IMPORTANT**: Copy both the certificate and private key

6. Save certificate to server:
```bash
# On your local machine
scp origin-cert.pem deploy@91.98.29.81:~/pixel-village/deploy/certs/
scp origin-key.pem deploy@91.98.29.81:~/pixel-village/deploy/certs/
```

7. Update Caddyfile to use origin certificates (see Caddyfile.origin-cert example)

## Always Use HTTPS

1. Go to SSL/TLS → Edge Certificates
2. Enable "Always Use HTTPS"
3. Enable "Automatic HTTPS Rewrites"

## WebSocket Support

WebSocket is enabled by default in Cloudflare. To verify:

1. Go to Network
2. Ensure "WebSockets" is ON

## Security Settings

### Firewall Rules (Optional but Recommended)

1. Go to Security → WAF → Custom rules
2. Create a new rule "Admin Panel Protection":
   - Field: Hostname
   - Operator: equals
   - Value: `admin.pixel-village.online`
   - Then: Challenge (Managed Challenge)
3. Save and Deploy

### Rate Limiting

1. Go to Security → WAF → Rate limiting rules
2. Create rule "Admin API Rate Limit":
   - If: `http.request.uri.path starts_with "/api/admin/"`
   - When rate exceeds: 30 requests per 1 minute
   - Then: Block
3. Save and Deploy

### Cloudflare Access (Optional - Extra Security for Admin)

For maximum security, you can put admin panel behind Cloudflare Access:

1. Go to Zero Trust → Access → Applications
2. Click "Add an application"
3. Select "Self-hosted"
4. Configure:
   - Application name: Pixel Village Admin
   - Session Duration: 24 hours
   - Application domain: `admin.pixel-village.online`
5. Add policy:
   - Policy name: Admin Access
   - Action: Allow
   - Include: Emails ending in `@yourdomain.com` or your specific email
6. Save

**Note**: With Cloudflare Access enabled, only authenticated users can access admin panel.

## Bot Protection

1. Go to Security → Bots
2. Enable "Bot Fight Mode" (Free plan)
   - Or "Super Bot Fight Mode" (Pro+ plan)

## DDoS Protection

DDoS protection is enabled by default. To configure:

1. Go to Security → DDoS
2. Review DDoS Managed Ruleset
3. All rules are active by default

## Verification

After configuration, test the endpoints:

```bash
# Test public endpoints
curl https://api.pixel-village.online/api/version
curl https://pixel-village.online/

# Test WebSocket (from game client)
# Connection should work to wss://api.pixel-village.online/ws

# Test admin panel (in browser)
# https://admin.pixel-village.online/admin
```

## Troubleshooting

### SSL/TLS errors
- Check SSL mode is "Full (strict)"
- Verify Caddy is running: `docker compose ps`
- Check Caddy logs: `docker compose logs caddy`

### 502 Bad Gateway
- Check if game server is running: `docker compose ps`
- Check server logs: `docker compose logs game-server`
- Verify port 8080 is accessible from Caddy container

### WebSocket not connecting
- Verify WebSocket is enabled in Cloudflare Network settings
- Check Caddyfile has proper WebSocket headers
- Test direct connection: `curl -i -N -H "Connection: Upgrade" -H "Upgrade: websocket" https://api.pixel-village.online/ws`

### Admin panel unauthorized
- Check ADMIN_KEY is set in .env
- Verify X-Admin-Key header is being sent
- Check rate limiting hasn't blocked your IP

## Next Steps

After Cloudflare is configured:
- Proceed to Stage 5: Client configuration (update PV_SERVER variable)
- Set up monitoring (UptimeRobot)
- Configure backups (Stage 7)
