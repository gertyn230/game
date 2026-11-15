#!/bin/bash
# Setup automatic daily backups via cron
# Run this script as 'deploy' user

set -e

echo "Setting up automatic daily backups..."

# Make backup script executable
chmod +x ~/pixel-village/deploy/backup-db.sh

# Add cron job (daily at 3 AM)
CRON_JOB="0 3 * * * /home/deploy/pixel-village/deploy/backup-db.sh >> /home/deploy/backups/backup.log 2>&1"

# Check if cron job already exists
if crontab -l 2>/dev/null | grep -q "backup-db.sh"; then
    echo "Cron job already exists"
else
    # Add to crontab
    (crontab -l 2>/dev/null; echo "$CRON_JOB") | crontab -
    echo "Cron job added successfully"
fi

echo ""
echo "Current crontab:"
crontab -l

echo ""
echo "Backup will run daily at 3:00 AM"
echo "Logs will be saved to: /home/deploy/backups/backup.log"
echo ""
echo "To view backup logs:"
echo "  tail -f ~/backups/backup.log"
echo ""
echo "To manually run backup:"
echo "  ~/pixel-village/deploy/backup-db.sh"
echo ""
