# Backup and Recovery

## What to Back Up

| Component | Location | Criticality |
|-----------|----------|-------------|
| Node identity (keypair) | `data_dir/identity.pem` or config | **Critical** — losing this means losing node identity |
| Proof database | `data_dir/proofs.duckdb` | High — contains all proof data |
| Analytics database | `data_dir/analytics.duckdb` | Medium — can be regenerated |
| Vector store embeddings | Chroma/Qdrant data directory | Medium — can be re-embedded |
| Configuration | `config.toml` | High — environment-specific settings |

## Backup Procedures

### 1. Keypair Backup (Critical)

The node identity keypair is the most important data to back up:

```bash
# Export existing keypair to PEM
poi-cli key export --output /backup/poi-node-identity.pem

# Or locate the identity file
ls -la /var/lib/poi/identity.pem

# Encrypt before storing off-site
gpg --symmetric --cipher-algo AES256 /backup/poi-node-identity.pem
```

### 2. Database Backup

#### DuckDB (Proof Database)

```bash
#!/bin/bash
BACKUP_DIR="/backup/poi"
DB_PATH="/var/lib/poi/data/proofs.duckdb"

mkdir -p "$BACKUP_DIR"

# Live backup using DuckDB's built-in backup
python -c "
import duckdb
conn = duckdb.connect('$DB_PATH')
conn.execute(f\"BACKUP DATABASE TO '{BACKUP_DIR}/proofs-$(date +%Y%m%d-%H%M%S).duckdb'\")
conn.close()
"

# Or use file copy (node must be stopped or in read-only mode)
# cp "$DB_PATH" "$BACKUP_DIR/proofs-$(date +%Y%m%d-%H%M%S).duckdb"
```

#### Analytics Database

```bash
cp /var/lib/poi/data/analytics.duckdb /backup/poi/analytics-$(date +%Y%m%d-%H%M%S).duckdb
```

### 3. Vector Store Backup

#### Chroma

```bash
# Stop Chroma first, then copy the data directory
docker stop chroma
tar czf /backup/poi/chroma-$(date +%Y%m%d-%H%M%S).tar.gz /chroma/data
docker start chroma
```

#### Qdrant

```bash
# Qdrant snapshots via API
curl -X POST 'http://localhost:6333/collections/poi-proofs/snapshots'
# Download from storage directory
```

### 4. Configuration Backup

```bash
cp /etc/poi/config.toml /backup/poi/config-$(date +%Y%m%d-%H%M%S).toml
```

## Automated Backup Script

```bash
#!/bin/bash
# /usr/local/bin/poi-backup.sh

set -euo pipefail

BACKUP_DIR="/backup/poi"
DB_PATH="/var/lib/poi/data"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
RETENTION_DAYS=30

mkdir -p "$BACKUP_DIR"

echo "Starting POI backup at $(date)"

# Keypair backup
if [ -f "$DB_PATH/identity.pem" ]; then
    gpg --symmetric --batch --pass-file /etc/poi/backup-passphrase \
        --output "$BACKUP_DIR/identity-$TIMESTAMP.pem.gpg" \
        "$DB_PATH/identity.pem"
    echo "  Keypair backed up"
fi

# Proof database
python3 -c "
import duckdb
conn = duckdb.connect('$DB_PATH/proofs.duckdb')
conn.execute(\"BACKUP DATABASE TO '$BACKUP_DIR/proofs-$TIMESTAMP.duckdb'\")
" && echo "  Proof database backed up"

# Analytics database
cp "$DB_PATH/analytics.duckdb" "$BACKUP_DIR/analytics-$TIMESTAMP.duckdb" \
    && echo "  Analytics database backed up"

# Config
cp /etc/poi/config.toml "$BACKUP_DIR/config-$TIMESTAMP.toml" \
    && echo "  Config backed up"

# Cleanup old backups
find "$BACKUP_DIR" -name "*.duckdb" -mtime +$RETENTION_DAYS -delete
find "$BACKUP_DIR" -name "*.pem.gpg" -mtime +$RETENTION_DAYS -delete
find "$BACKUP_DIR" -name "*.toml" -mtime +$RETENTION_DAYS -delete

echo "Backup complete at $(date)"
```

### Cron Job

```bash
# Run daily at 2 AM
0 2 * * * /usr/local/bin/poi-backup.sh >> /var/log/poi-backup.log 2>&1
```

## Recovery Procedures

### Full Node Recovery

```bash
# 1. Install POI node
cargo build --release -p poi-node

# 2. Restore configuration
cp /backup/poi/config-20260523.toml /etc/poi/config.toml

# 3. Restore identity
gpg --decrypt /backup/poi/identity-20260523.pem.gpg > /var/lib/poi/data/identity.pem

# 4. Restore databases
cp /backup/poi/proofs-20260523.duckdb /var/lib/poi/data/proofs.duckdb
cp /backup/poi/analytics-20260523.duckdb /var/lib/poi/data/analytics.duckdb

# 5. Start node
systemctl start poi-node
```

### Partial Recovery

#### Lost Database but Have Keypair

```bash
# Start fresh with existing identity
poi-cli key import /var/lib/poi/data/identity.pem
systemctl start poi-node
# Proofs will be recreated through normal operation
```

#### Lost Keypair

```bash
# Generate new keypair
poi-cli key generate --output /var/lib/poi/data/identity.pem

# New identity means old proofs cannot be attributed to this node
# Old proofs will show as "unknown node"
```

## Backup Verification

```bash
# Test that a backup is valid
python3 -c "
import duckdb
conn = duckdb.connect('/backup/poi/proofs-20260523.duckdb')
rows = conn.execute('SELECT COUNT(*) FROM proofs').fetchone()
print(f'Backup contains {rows[0]} proofs')
"
```

## Disaster Recovery Plan

1. **Node failure**: Restart the service (`systemctl restart poi-node`)
2. **Database corruption**: Restore from most recent backup
3. **Hardware failure**: Provision new server, restore from backup
4. **Full datacenter loss**: Restore from off-site backup (S3/Glacier)
5. **Key compromise**: Generate new keypair, revoke old public key across network

## Backup Storage Recommendations

| Tier | Storage | Retention | Use Case |
|------|---------|-----------|----------|
| Hot | Local disk | 7 days | Quick recovery |
| Warm | S3/Wasabi | 30 days | Regular restore |
| Cold | S3 Glacier | 1 year | Compliance/audit |
