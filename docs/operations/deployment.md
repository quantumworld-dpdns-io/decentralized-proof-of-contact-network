# Deployment Guide

## Production Architecture

```
                    ┌─────────────────────┐
                    │   Load Balancer     │
                    │   (HAProxy / Nginx) │
                    └──────┬──────────────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
        ┌─────┴────┐ ┌─────┴────┐ ┌─────┴────┐
        │  Node 1  │ │  Node 2  │ │  Node N  │
        │ :3000    │ │ :3000    │ │ :3000    │
        └──────────┘ └──────────┘ └──────────┘
              │            │            │
              └────────────┼────────────┘
                           │
              ┌────────────┴────────────┐
              │   Shared Storage / DB   │
              │   (PostgreSQL / S3)     │
              └─────────────────────────┘
```

## Prerequisites

- Rust toolchain (for building from source) or Docker
- Postgres-compatible database (for production storage)
- S3-compatible object storage (for Iceberg analytics)
- TLS certificates for API and P2P communication
- Monitoring stack (Prometheus + Grafana)

## Docker Deployment

### Build Images

```bash
# Node image
make docker-node

# API image
make docker-api

# All images
make docker-all
```

### Docker Compose

```yaml
# docker-compose.yml (production)
version: "3.8"
services:
  poi-node:
    image: poi-node:latest
    ports:
      - "3000:3000"   # REST API
      - "9090:9090"   # P2P
      - "9091:9091"   # Metrics
    environment:
      - NODE_ID=${NODE_ID}
      - EXTERNAL_ADDR=${EXTERNAL_ADDR}
      - VECTOR_DB_PROVIDER=chroma
      - VECTOR_DB_HOST=chroma
      - VECTOR_DB_PORT=8000
      - AI_PROVIDER=ollama
      - AI_MODEL=gemma3:latest
      - AI_ENDPOINT=http://ollama:11434
      - OTLP_ENDPOINT=http://otel-collector:4317
    volumes:
      - poi-data:/var/lib/poi
      - ./tls:/etc/poi/tls:ro
    configs:
      - source: poi-config
        target: /etc/poi/config.toml
    depends_on:
      - chroma
      - ollama

  chroma:
    image: chromadb/chroma:latest
    volumes:
      - chroma-data:/chroma/chroma

  ollama:
    image: ollama/ollama:latest
    volumes:
      - ollama-models:/root/.ollama
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: all
              capabilities: [gpu]

  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./docker/prometheus:/etc/prometheus
      - prometheus-data:/prometheus

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    volumes:
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards
      - grafana-data:/var/lib/grafana

volumes:
  poi-data:
  chroma-data:
  ollama-models:
  prometheus-data:
  grafana-data:

configs:
  poi-config:
    file: ./config.prod.toml
```

### Start

```bash
docker compose -f docker/docker-compose.yml up -d
```

## Manual Deployment (Bare Metal)

### 1. Build Release Binaries

```bash
cargo build --workspace --release
```

Binaries are at `target/release/`:
- `poi-node` — Full node
- `poi-cli` — CLI tool
- `poi-mcp-server` — MCP server

### 2. Create System User

```bash
sudo useradd -r -s /bin/false -m -d /var/lib/poi poi
```

### 3. Set Up Directory Structure

```bash
sudo mkdir -p /var/lib/poi/{data,db,logs,tls}
sudo chown -R poi:poi /var/lib/poi
```

### 4. Configure

```bash
sudo cp config.prod.toml /etc/poi/config.toml
sudo chown poi:poi /etc/poi/config.toml
sudo chmod 600 /etc/poi/config.toml
```

### 5. Systemd Service

```ini
# /etc/systemd/system/poi-node.service
[Unit]
Description=POI Network Node
After=network.target

[Service]
Type=simple
User=poi
Group=poi
WorkingDirectory=/var/lib/poi
ExecStart=/usr/local/bin/poi-node --config /etc/poi/config.toml
Restart=always
RestartSec=5
LimitNOFILE=65536

Environment="NODE_ID=%H"
Environment="EXTERNAL_ADDR=%H:9090"
Environment="RUST_LOG=info"
Environment="RUST_BACKTRACE=1"

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable poi-node
sudo systemctl start poi-node
sudo systemctl status poi-node
```

### 6. TLS Certificates

For production, configure TLS for the REST API:

```toml
[api]
bind_addr = "0.0.0.0:3000"
tls_cert = "/etc/poi/tls/cert.pem"
tls_key = "/etc/poi/tls/key.pem"
```

Generate with Let's Encrypt:

```bash
sudo certbot certonly --standalone -d poi-node.example.com
sudo ln -s /etc/letsencrypt/live/poi-node.example.com/fullchain.pem /etc/poi/tls/cert.pem
sudo ln -s /etc/letsencrypt/live/poi-node.example.com/privkey.pem /etc/poi/tls/key.pem
```

## Environment-Specific Configuration

### Development

```toml
[node]
log_level = "debug"
data_dir = "./data"
```

### Staging

```toml
[node]
log_level = "debug"
data_dir = "/var/lib/poi"
```

### Production

```toml
[node]
log_level = "info"
data_dir = "/var/lib/poi"
enable_pqc = false  # Enable only after thorough testing
```

## Health Checks

Configure your load balancer to check:

```
GET /api/v1/health
```

Expected: `{"status": "ok"}`

## Backup

See [Backup and Recovery](backup.md) for detailed backup procedures.
