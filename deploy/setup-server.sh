#!/bin/bash
# Pixel Village Server Setup - Stage 2: Server Configuration
# Run this script on Ubuntu 24.04 ARM64 server as root or with sudo

set -e  # Exit on error

echo "==================================="
echo "Pixel Village Server Setup - Stage 2"
echo "==================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if running as root
if [ "$EUID" -ne 0 ]; then
   echo -e "${RED}Please run as root or with sudo${NC}"
   exit 1
fi

echo -e "${GREEN}[1/8] Updating system...${NC}"
apt update && apt upgrade -y

echo -e "${GREEN}[2/8] Installing essential packages...${NC}"
apt install -y \
    curl \
    wget \
    git \
    unattended-upgrades \
    ufw \
    fail2ban \
    ca-certificates \
    gnupg \
    lsb-release

# Configure automatic security updates
echo -e "${GREEN}[3/8] Configuring automatic security updates...${NC}"
cat > /etc/apt/apt.conf.d/50unattended-upgrades <<EOF
Unattended-Upgrade::Allowed-Origins {
    "\${distro_id}:\${distro_codename}-security";
    "\${distro_id}ESMApps:\${distro_codename}-apps-security";
    "\${distro_id}ESM:\${distro_codename}-infra-security";
};
Unattended-Upgrade::AutoFixInterruptedDpkg "true";
Unattended-Upgrade::MinimalSteps "true";
Unattended-Upgrade::Remove-Unused-Kernel-Packages "true";
Unattended-Upgrade::Remove-Unused-Dependencies "true";
Unattended-Upgrade::Automatic-Reboot "false";
EOF

systemctl enable unattended-upgrades
systemctl start unattended-upgrades

echo -e "${GREEN}[4/8] Installing Docker...${NC}"
# Add Docker's official GPG key
install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
chmod a+r /etc/apt/keyrings/docker.asc

# Add the repository to Apt sources
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu \
  $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
  tee /etc/apt/sources.list.d/docker.list > /dev/null

apt update
apt install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

# Start and enable Docker
systemctl enable docker
systemctl start docker

echo -e "${GREEN}[5/8] Creating non-root user 'deploy'...${NC}"
if ! id -u deploy > /dev/null 2>&1; then
    useradd -m -s /bin/bash deploy
    usermod -aG docker deploy
    echo "User 'deploy' created and added to docker group"
else
    echo "User 'deploy' already exists"
fi

echo -e "${GREEN}[6/8] Configuring SSH security...${NC}"
# Backup original sshd_config
cp /etc/ssh/sshd_config /etc/ssh/sshd_config.backup

# Configure SSH
sed -i 's/#PermitRootLogin yes/PermitRootLogin no/' /etc/ssh/sshd_config
sed -i 's/PermitRootLogin yes/PermitRootLogin no/' /etc/ssh/sshd_config
sed -i 's/#PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config
sed -i 's/PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config
sed -i 's/#PubkeyAuthentication yes/PubkeyAuthentication yes/' /etc/ssh/sshd_config

# Restart SSH
systemctl restart sshd

echo -e "${GREEN}[7/8] Configuring firewall (UFW)...${NC}"
# Reset UFW to default
ufw --force reset

# Default policies
ufw default deny incoming
ufw default allow outgoing

# Allow SSH (important - do this first!)
ufw allow 22/tcp comment 'SSH'

# Allow HTTP and HTTPS
ufw allow 80/tcp comment 'HTTP'
ufw allow 443/tcp comment 'HTTPS'

# Enable UFW
ufw --force enable

echo -e "${GREEN}[8/8] Configuring fail2ban...${NC}"
cat > /etc/fail2ban/jail.local <<EOF
[DEFAULT]
bantime = 3600
findtime = 600
maxretry = 5
backend = systemd

[sshd]
enabled = true
port = 22
logpath = /var/log/auth.log
maxretry = 3
EOF

systemctl enable fail2ban
systemctl start fail2ban

echo ""
echo -e "${GREEN}==================================="
echo "Stage 2 Complete!"
echo "===================================${NC}"
echo ""
echo -e "${YELLOW}Important next steps:${NC}"
echo "1. Add your SSH public key to /home/deploy/.ssh/authorized_keys"
echo "2. Test SSH login as 'deploy' user before closing this session"
echo "3. Configure Hetzner Cloud Firewall to limit access"
echo "4. Proceed to Stage 3: Deploy application"
echo ""
echo -e "${YELLOW}Current status:${NC}"
docker --version
docker compose version
ufw status
systemctl status fail2ban --no-pager
echo ""
