# Stage 5: Client Configuration and Auto-Update

## Update GitHub Actions Variable

The game clients need to know the server address. This is configured via GitHub Actions variable `PV_SERVER`.

### Step 1: Check Current Client Code

First, check what format the client expects:

```bash
cd ~/game-deploy
grep -n "PV_SERVER" client/src/main.rs
```

The client likely expects one of these formats:
- `wss://api.pixel-village.online` (WebSocket with path)
- `wss://api.pixel-village.online/ws` (WebSocket with explicit /ws)
- `api.pixel-village.online` (domain only, client adds wss:// and /ws)

### Step 2: Set GitHub Actions Variable

1. Go to GitHub repository: https://github.com/gertyn230/game
2. Click "Settings" → "Secrets and variables" → "Actions"
3. Click "Variables" tab
4. Find `PV_SERVER` variable (or create new if doesn't exist)
5. Set value based on what client expects (most likely):
   ```
   wss://api.pixel-village.online/ws
   ```
6. Click "Update variable" or "Add variable"

### Step 3: Trigger Client Rebuild

The clients need to be rebuilt with the new server address:

1. Go to "Actions" tab in GitHub
2. Find "Build" workflow
3. Click "Run workflow" → "Run workflow"
4. Wait for builds to complete (Windows, Android, macOS, iOS)

All clients will be rebuilt with the new production server address.

### Step 4: Test Auto-Update System

The game has auto-update functionality that checks `/api/version` endpoint.

1. **Test version endpoint:**
   ```bash
   curl https://api.pixel-village.online/api/version
   ```
   
   Should return:
   ```json
   {
     "version": "0.1.0",
     "date": "2026-11-07",
     "changes": [...],
     "download": "https://github.com/gertyn230/game/releases/latest"
   }
   ```

2. **How auto-update works:**
   - Client checks `/api/version` on startup
   - Compares with built-in version
   - If newer version available, shows update prompt
   - Downloads from GitHub Releases

3. **Update process for users:**
   - Client shows: "New version X.X.X available"
   - User clicks "Update"
   - Client downloads from GitHub Releases
   - Client restarts with new version

### Step 5: Update Version (When Releasing New Version)

When you want to release a new version:

1. **Update version in code:**
   ```bash
   cd ~/game-deploy
   # Edit Cargo.toml
   nano Cargo.toml
   # Update version = "0.1.0" to new version
   ```

2. **Update version info in server:**
   ```bash
   nano server/src/main.rs
   # Find api_version() function
   # Update version, date, and changes list
   ```

3. **Commit and push:**
   ```bash
   git add Cargo.toml server/src/main.rs
   git commit -m "Release version X.X.X"
   git push origin main
   ```

4. **Create GitHub Release:**
   - Go to GitHub → Releases → "Draft a new release"
   - Tag version: `vX.X.X`
   - Release title: `Version X.X.X`
   - Description: Copy changes from api_version()
   - Wait for builds to complete
   - Publish release

5. **Deploy to production server:**
   ```bash
   ssh deploy@91.98.29.81
   cd ~/pixel-village
   git pull origin main
   cd deploy
   docker compose up -d --build
   ```

6. **Verify clients can auto-update:**
   - Open old client version
   - Client should detect new version
   - Click "Update"
   - Client downloads and restarts

### Step 6: Verify Everything Works

Test the complete flow:

1. **Server is accessible:**
   ```bash
   curl https://api.pixel-village.online/api/version
   ```

2. **WebSocket works:**
   - Open game client
   - Try to connect
   - Should connect to `wss://api.pixel-village.online/ws`

3. **Admin panel works:**
   - Open https://admin.pixel-village.online/admin
   - Enter ADMIN_KEY
   - Should load admin panel

4. **Auto-update works:**
   - Old client detects new version
   - Downloads from GitHub Releases
   - Updates successfully

## Important Notes

### Hardcoded Version in api_version()

The version info in `server/src/main.rs` → `api_version()` function is hardcoded.

**Current implementation:**
```rust
"version": env!("CARGO_PKG_VERSION"),  // This reads from Cargo.toml
"date": "2026-11-07",                  // ⚠️ Hardcoded - update manually
"changes": [                            // ⚠️ Hardcoded - update manually
    "Feature 1",
    "Feature 2"
],
```

**When releasing new version:**
1. Update version in `Cargo.toml` (auto-read by env!("CARGO_PKG_VERSION"))
2. Manually update `date` in `api_version()`
3. Manually update `changes` array

### Client Configuration

The client gets server address from compile-time environment variable `PV_SERVER`.

**In GitHub Actions (`.github/workflows/build.yml`):**
```yaml
env:
  PV_SERVER: wss://api.pixel-village.online/ws
```

**In client code (`client/src/main.rs`):**
```rust
const SERVER_URL: &str = env!("PV_SERVER");
```

This means:
- Clients are compiled with server address baked in
- To change server, must rebuild all clients
- Cannot change server address after build

### game-releases Repository

Check if you have a separate repository for releases:
- Repository name: `gertyn230/game-releases` (mentioned in memory)
- Purpose: May store release assets or website
- Action: Verify if this exists and what it contains

## Troubleshooting

### Client can't connect to server
- Check `PV_SERVER` variable is correct
- Verify WebSocket is enabled in Cloudflare
- Test WebSocket: `wscat -c wss://api.pixel-village.online/ws`

### Auto-update not working
- Check `/api/version` returns correct version
- Verify GitHub Release has built assets
- Check client has internet connection
- Verify download URL is accessible

### Version mismatch
- Ensure `Cargo.toml` version matches
- Update hardcoded date and changes in `api_version()`
- Rebuild and redeploy server

## Next Steps

After clients are configured:
- Test full gameplay loop
- Verify auto-update works
- Configure website (Stage 6)
- Set up monitoring (Stage 7)
