#!/bin/bash
# Pixel Village - Complete Auto Deploy Script
# Just run: bash deploy.sh

set -e

echo "=================================="
echo "🚀 PIXEL VILLAGE AUTO DEPLOY"
echo "=================================="
echo ""

# Update and install packages
echo "[1/8] Installing packages..."
apt update
apt install -y curl git docker.io docker-compose sqlite3

# Start Docker
echo "[2/8] Starting Docker..."
systemctl start docker
systemctl enable docker

# Create deploy user
echo "[3/8] Creating deploy user..."
useradd -m -s /bin/bash deploy 2>/dev/null || echo "User exists"
usermod -aG docker deploy

# Clone repository
echo "[4/8] Cloning repository..."
cd /home/deploy
if [ -d "game" ]; then
    cd game && git pull
else
    git clone https://github.com/gertyn230/game.git game
    cd game
fi

# Setup environment
echo "[5/8] Setting up environment..."
cd deploy
cp ../.env.example .env
ADMIN_KEY=$(openssl rand -hex 32)
echo "" >> .env
echo "ADMIN_KEY=$ADMIN_KEY" >> .env

# Create data directory
echo "[6/8] Creating data directory..."
mkdir -p /var/lib/pixel-village/data
chown -R deploy:deploy /var/lib/pixel-village /home/deploy/game

# Build and deploy
echo "[7/8] Building application..."
su - deploy -c "cd ~/game/deploy && docker compose build"

echo "[8/8] Starting application..."
su - deploy -c "cd ~/game/deploy && docker compose up -d"

# Show results
echo ""
echo "=================================="
echo "✅ DEPLOYMENT COMPLETE!"
echo "=================================="
echo ""
echo "🔑 ADMIN KEY (SAVE THIS!):"
echo "   $ADMIN_KEY"
echo ""
echo "📊 Service status:"
su - deploy -c "cd ~/game/deploy && docker compose ps"
echo ""
echo "🌐 Next steps:"
echo "   1. Configure DNS: api.pixel-village.online -> 91.98.29.81"
echo "   2. Wait 2-3 minutes for services to start"
echo "   3. Test: curl http://localhost:8080/api/version"
echo ""
