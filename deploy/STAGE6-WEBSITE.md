# Stage 6: Website Deployment

The main website for `pixel-village.online` and `www.pixel-village.online`.

## Check if Website Code Exists

Based on memory, you have a separate repository `gertyn230/game-releases` for the website.

### Step 1: Verify Repository

```bash
# Check if repository exists
# Visit: https://github.com/gertyn230/game-releases
```

If the repository exists and contains website code (Next.js, HTML, etc.), proceed with deployment.

## Option A: Cloudflare Pages (Recommended for Static Sites)

Best for: Static sites, Next.js with static export, React, Vue, etc.

### Setup Steps:

1. **Prepare Website for Static Export** (if using Next.js)
   
   In `game-releases` repository, ensure `next.config.js` has:
   ```javascript
   module.exports = {
     output: 'export',
     images: {
       unoptimized: true,
     },
   }
   ```

2. **Connect to Cloudflare Pages:**
   - Go to Cloudflare Dashboard → Pages
   - Click "Create a project"
   - Click "Connect to Git"
   - Select your GitHub account
   - Choose `gertyn230/game-releases` repository
   - Configure build settings:
     - Framework preset: `Next.js` (or appropriate)
     - Build command: `npm run build` (or `npm run export`)
     - Build output directory: `out` (Next.js) or `dist` (Vite)
   - Click "Save and Deploy"

3. **Configure Custom Domain:**
   - After first deploy, go to project settings
   - Click "Custom domains"
   - Add domains:
     - `pixel-village.online`
     - `www.pixel-village.online`
   - Cloudflare will automatically configure DNS

4. **Set Environment Variables** (if needed):
   - Go to project settings → Environment variables
   - Add any required variables:
     - `NEXT_PUBLIC_API_URL=https://api.pixel-village.online`
     - etc.

### Advantages of Cloudflare Pages:
- ✅ Free hosting
- ✅ Automatic HTTPS
- ✅ Global CDN
- ✅ Auto-deploy on git push
- ✅ Preview deployments for PRs
- ✅ Zero server management

## Option B: Deploy on Same Server

If you want to host website on your Hetzner server:

### Setup with Nginx:

1. **Build website locally:**
   ```bash
   cd ~/game-releases
   npm install
   npm run build
   # Output will be in 'out/' or 'dist/'
   ```

2. **Upload to server:**
   ```bash
   scp -r out/* deploy@91.98.29.81:~/website/
   ```

3. **Update Caddyfile:**
   ```caddyfile
   pixel-village.online, www.pixel-village.online {
       root * /home/deploy/website
       file_server
       
       # SPA routing (if using React Router, etc.)
       try_files {path} /index.html
       
       # Security headers
       header {
           -Server
           X-Content-Type-Options "nosniff"
           X-Frame-Options "DENY"
           X-XSS-Protection "1; mode=block"
       }
       
       log {
           output file /var/log/caddy/www.log
           format json
       }
   }
   ```

4. **Restart Caddy:**
   ```bash
   docker compose restart caddy
   ```

### Advantages of Self-Hosting:
- ✅ Full control
- ✅ Can use dynamic server-side features
- ❌ Need to manage server
- ❌ Manual deploys

## Option C: No Website Yet

If website doesn't exist yet, the Caddyfile already has a placeholder:

```caddyfile
pixel-village.online, www.pixel-village.online {
    respond "Pixel Village - Coming Soon" 200
}
```

You can enhance it with a simple HTML page:

```bash
# On server
mkdir -p ~/website
cat > ~/website/index.html <<'EOF'
<!DOCTYPE html>
<html lang="uk">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Pixel Village - Coming Soon</title>
    <style>
        body {
            margin: 0;
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            font-family: system-ui, -apple-system, sans-serif;
            color: white;
            text-align: center;
            padding: 20px;
        }
        .container {
            max-width: 600px;
        }
        h1 {
            font-size: 3rem;
            margin: 0 0 1rem;
        }
        p {
            font-size: 1.25rem;
            opacity: 0.9;
            margin: 0 0 2rem;
        }
        .links {
            display: flex;
            gap: 1rem;
            justify-content: center;
            flex-wrap: wrap;
        }
        a {
            padding: 0.75rem 1.5rem;
            background: white;
            color: #667eea;
            text-decoration: none;
            border-radius: 8px;
            font-weight: 600;
            transition: transform 0.2s;
        }
        a:hover {
            transform: scale(1.05);
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🎮 Pixel Village</h1>
        <p>Ізометричний світ пригод незабаром відкриється!</p>
        <div class="links">
            <a href="https://github.com/gertyn230/game/releases/latest">Завантажити гру</a>
            <a href="https://admin.pixel-village.online/admin">Адмін-панель</a>
        </div>
    </div>
</body>
</html>
EOF
```

Update Caddyfile:
```caddyfile
pixel-village.online, www.pixel-village.online {
    root * /home/deploy/website
    file_server
}
```

## Recommended Approach

**For now**: Use Option C (simple coming soon page)
**Long term**: Deploy actual website using Cloudflare Pages

## Testing

After deployment:
```bash
curl https://pixel-village.online/
curl https://www.pixel-village.online/
```

Both should return website content.

## Next Steps

After website is deployed:
- Proceed to Stage 7: Monitoring and backups
- Set up analytics (Google Analytics, Plausible, etc.)
- Add SEO metadata
- Configure sitemap.xml and robots.txt
