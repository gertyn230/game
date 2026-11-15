#!/bin/bash
# Pixel Village Database Restore Script
# Restores database from backup

set -e

BACKUP_DIR="/home/deploy/backups"
DB_PATH="/var/lib/pixel-village/data/game.db"

echo "==================================="
echo "Pixel Village Database Restore"
echo "==================================="
echo ""

# List available backups
echo "Available backups:"
ls -lht "$BACKUP_DIR"/game-backup-*.db.gz 2>/dev/null || {
    echo "No backups found in $BACKUP_DIR"
    exit 1
}

echo ""
read -p "Enter backup filename to restore (or 'latest' for most recent): " BACKUP_CHOICE

if [ "$BACKUP_CHOICE" = "latest" ]; then
    BACKUP_FILE=$(ls -t "$BACKUP_DIR"/game-backup-*.db.gz 2>/dev/null | head -1)
else
    BACKUP_FILE="$BACKUP_DIR/$BACKUP_CHOICE"
fi

if [ ! -f "$BACKUP_FILE" ]; then
    echo "Error: Backup file not found: $BACKUP_FILE"
    exit 1
fi

echo ""
echo "Selected backup: $BACKUP_FILE"
echo "This will replace current database at: $DB_PATH"
echo ""
read -p "Are you sure? This cannot be undone! (yes/no): " CONFIRM

if [ "$CONFIRM" != "yes" ]; then
    echo "Restore cancelled"
    exit 0
fi

echo ""
echo "Stopping game server..."
cd ~/pixel-village/deploy
docker compose stop game-server

echo "Creating safety backup of current database..."
SAFETY_BACKUP="$BACKUP_DIR/pre-restore-$(date +%Y%m%d-%H%M%S).db"
if [ -f "$DB_PATH" ]; then
    cp "$DB_PATH" "$SAFETY_BACKUP"
    echo "Safety backup created: $SAFETY_BACKUP"
fi

echo "Restoring from backup..."
gunzip -c "$BACKUP_FILE" > "$DB_PATH"

echo "Setting correct permissions..."
sudo chown deploy:deploy "$DB_PATH"

echo "Starting game server..."
docker compose start game-server

echo ""
echo "Waiting for server to start..."
sleep 5

echo "Checking server status..."
docker compose ps game-server

echo ""
echo "==================================="
echo "Restore completed!"
echo "==================================="
echo ""
echo "Verify that everything works correctly."
echo "If there are issues, you can restore from safety backup:"
echo "  $SAFETY_BACKUP"
echo ""
