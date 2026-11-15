#!/bin/bash
# Pixel Village Server Setup - Stage 3: Deploy Application
# Run this script as 'deploy' user on the server

set -e

echo "==================================="
echo "Pixel Village Deploy - Stage 3"
echo "==================================="
echo ""

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check if running as deploy user
if [ "$(whoami)" != "deploy" ]; then
    echo "Please run this script as 'deploy' user"
    echo "Switch to deploy user: sudo su - deploy"
    exit 1
fi

cd ~

echo -e "${GREEN}[1/6] Cloning repository...${NC}"
if [ -d "pixel-village" ]; then
    echo "Directory exists, pulling latest changes..."
    cd pixel-village
    git pull origin main
else
    git clone https://github.com/gertyn230/game.git pixel-village
    cd pixel-village
fi

echo -e "${GREEN}[2/6] Setting up environment...${NC}"
cd deploy

if [ ! -f ".env" ]; then
    echo "Creating .env file from template..."
    cp ../.env.example .env

    echo ""
    echo -e "${YELLOW}IMPORTANT: You need to configure .env file${NC}"
    echo "Please edit .env and set:"
    echo "  - ADMIN_KEY (use: openssl rand -hex 32)"
    echo ""
    read -p "Press Enter to edit .env now..."
    nano .env
else
    echo ".env file already exists"
fi

# Validate that ADMIN_KEY is set
if ! grep -q "^ADMIN_KEY=.\+" .env; then
    echo ""
    echo -e "${YELLOW}WARNING: ADMIN_KEY not set in .env${NC}"
    echo "Generating random ADMIN_KEY..."
    ADMIN_KEY=$(openssl rand -hex 32)
    echo "ADMIN_KEY=$ADMIN_KEY" >> .env
    echo "Generated ADMIN_KEY: $ADMIN_KEY"
    echo "SAVE THIS KEY! You'll need it to access admin panel."
    echo ""
    read -p "Press Enter to continue..."
fi

echo -e "${GREEN}[3/6] Building Docker image...${NC}"
docker compose build --no-cache

echo -e "${GREEN}[4/6] Creating data directory...${NC}"
sudo mkdir -p /var/lib/pixel-village/data
sudo chown -R deploy:deploy /var/lib/pixel-village

echo -e "${GREEN}[5/6] Starting services...${NC}"
docker compose up -d

echo -e "${GREEN}[6/6] Checking service health...${NC}"
sleep 5
docker compose ps
docker compose logs --tail=50 game-server

echo ""
echo -e "${GREEN}==================================="
echo "Deployment Complete!"
echo "===================================${NC}"
echo ""
echo "Services status:"
docker compose ps
echo ""
echo "Test endpoints:"
echo "  curl http://localhost:8080/api/version"
echo ""
echo "View logs:"
echo "  docker compose logs -f game-server"
echo "  docker compose logs -f caddy"
echo ""
echo "Next: Configure DNS and SSL (Stage 4)"
echo ""
