#!/bin/bash
# Add SSH public key to deploy user
# Run this script as root after setup-server.sh

set -e

echo "Adding SSH public key to deploy user..."

# Check if running as root
if [ "$EUID" -ne 0 ]; then
   echo "Please run as root or with sudo"
   exit 1
fi

# Create .ssh directory for deploy user
mkdir -p /home/deploy/.ssh
chmod 700 /home/deploy/.ssh

# Add your SSH public key here (replace with your actual public key)
SSH_PUBLIC_KEY="ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAICTCI0JUclqFNr5b6HoTEANYhVddAUF2KwWhw5oAJEkZ pixel-village"

# Add key to authorized_keys
echo "$SSH_PUBLIC_KEY" > /home/deploy/.ssh/authorized_keys
chmod 600 /home/deploy/.ssh/authorized_keys
chown -R deploy:deploy /home/deploy/.ssh

echo "SSH key added successfully!"
echo "You can now login as: ssh deploy@91.98.29.81"
