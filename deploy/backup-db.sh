#!/bin/bash
# Pixel Village Database Backup Script
# Creates SQLite backup and stores it with retention policy

set -e

BACKUP_DIR="/home/deploy/backups"
DB_PATH="/var/lib/pixel-village/data/game.db"
RETENTION_DAYS=14
DATE=$(date +%Y%m%d-%H%M%S)
BACKUP_FILE="$BACKUP_DIR/game-backup-$DATE.db"

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

echo "Starting backup at $(date)"

# Check if database exists
if [ ! -f "$DB_PATH" ]; then
    echo "Error: Database not found at $DB_PATH"
    exit 1
fi

# Create backup using SQLite .backup command (safe for live database)
sqlite3 "$DB_PATH" ".backup '$BACKUP_FILE'"

# Compress backup
gzip "$BACKUP_FILE"
BACKUP_FILE="${BACKUP_FILE}.gz"

# Verify backup
if [ -f "$BACKUP_FILE" ]; then
    SIZE=$(du -h "$BACKUP_FILE" | cut -f1)
    echo "Backup created successfully: $BACKUP_FILE ($SIZE)"
else
    echo "Error: Backup failed"
    exit 1
fi

# Remove old backups (keep only last $RETENTION_DAYS days)
echo "Cleaning up old backups (keeping last $RETENTION_DAYS days)..."
find "$BACKUP_DIR" -name "game-backup-*.db.gz" -mtime +$RETENTION_DAYS -delete

# List current backups
echo ""
echo "Current backups:"
ls -lh "$BACKUP_DIR"/game-backup-*.db.gz 2>/dev/null || echo "No backups found"

echo ""
echo "Backup completed at $(date)"
