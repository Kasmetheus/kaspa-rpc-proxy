# Deployment Guide

## Quick Start (5 minutes)

### Prerequisites

- Docker 20.10+
- Docker Compose 2.0+
- 4GB RAM minimum
- 10GB disk space

### Steps

1. **Clone/Navigate**:
   ```bash
   cd kaspa-rpc-service
   ```

2. **Configure**:
   ```bash
   cp .env.example .env
   # Edit .env and change JWT_SECRET
   nano .env
   ```

3. **Start**:
   ```bash
   docker-compose up -d
   ```

4. **Wait for sync** (Kaspa node initialization):
   ```bash
   docker-compose logs -f kaspad
   # Wait until you see "Node is synced"
   # This may take 10-30 minutes on testnet
   ```

5. **Test**:
   ```bash
   ./tests/health_check.sh
   ./tests/test_dag_tips.sh
   ```

6. **Success!**
   Your RPC service is now running on `http://localhost:8080`

## Production Deployment

### Option 1: Docker Compose (Small Scale)

**server.yml**:
```yaml
version: '3.8'

services:
  kaspad:
    image: kaspanet/kaspad:latest
    command:
      - "--testnet"  # or remove for mainnet
      - "--rpclisten=0.0.0.0:16110"
      - "--utxoindex"
    volumes:
      - /data/kaspad:/app/data
    restart: always
    
  kaspa-rpc:
    image: your-registry/kaspa-rpc-service:latest
    environment:
      - KASPA_RPC_URL=http://kaspad:16110
      - JWT_SECRET=${JWT_SECRET}
    ports:
      - "8080:8080"
    depends_on:
      - kaspad
    restart: always
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 1G

  nginx:
    image: nginx:alpine
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - /etc/letsencrypt:/etc/letsencrypt:ro
    ports:
      - "443:443"
      - "80:80"
    depends_on:
      - kaspa-rpc
    restart: always
```

**nginx.conf** (TLS termination):
```nginx
server {
    listen 443 ssl http2;
    server_name api.your-domain.com;

    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;

    location / {
        proxy_pass http://kaspa-rpc:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### Option 2: Kubernetes (Large Scale)

**deployment.yaml**:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: kaspa-rpc-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: kaspa-rpc
  template:
    metadata:
      labels:
        app: kaspa-rpc
    spec:
      containers:
      - name: kaspa-rpc
        image: your-registry/kaspa-rpc-service:latest
        env:
        - name: KASPA_RPC_URL
          value: "http://kaspad-service:16110"
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: kaspa-secrets
              key: jwt-secret
        ports:
        - containerPort: 8080
        resources:
          requests:
            cpu: 500m
            memory: 512Mi
          limits:
            cpu: 2000m
            memory: 1Gi
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: kaspa-rpc-service
spec:
  type: LoadBalancer
  ports:
  - port: 80
    targetPort: 8080
  selector:
    app: kaspa-rpc
```

### Option 3: Systemd (Bare Metal)

**1. Build**:
```bash
cargo build --release
sudo cp target/release/kaspa-rpc-service /usr/local/bin/
```

**2. Create systemd service** (`/etc/systemd/system/kaspa-rpc.service`):
```ini
[Unit]
Description=Kaspa RPC Service
After=network.target kaspad.service
Requires=kaspad.service

[Service]
Type=simple
User=kaspa
Group=kaspa
WorkingDirectory=/opt/kaspa-rpc
Environment="KASPA_RPC_URL=http://localhost:16110"
Environment="JWT_SECRET=your-secret-here"
Environment="RUST_LOG=kaspa_rpc_service=info"
ExecStart=/usr/local/bin/kaspa-rpc-service
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/kaspa-rpc

[Install]
WantedBy=multi-user.target
```

**3. Enable and start**:
```bash
sudo systemctl daemon-reload
sudo systemctl enable kaspa-rpc
sudo systemctl start kaspa-rpc
sudo systemctl status kaspa-rpc
```

## Monitoring Setup

### Prometheus

**prometheus.yml**:
```yaml
scrape_configs:
  - job_name: 'kaspa-rpc'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

### Grafana Dashboard

Import from `monitoring/grafana-dashboard.json` (TODO)

## Backup Strategy

### What to Backup

1. **Configuration**:
   - `.env` file (encrypted)
   - `docker-compose.yml`
   - Custom scripts

2. **Kaspa Node Data** (optional):
   - `/data/kaspad` volume
   - Note: Can re-sync from network

3. **Logs** (optional):
   - Application logs
   - Access logs

### Backup Script

```bash
#!/bin/bash
# backup.sh

BACKUP_DIR=/backups/kaspa-rpc-$(date +%Y%m%d)
mkdir -p $BACKUP_DIR

# Backup config
cp .env $BACKUP_DIR/
cp docker-compose.yml $BACKUP_DIR/

# Backup Kaspa data (optional, large)
# tar czf $BACKUP_DIR/kaspad-data.tar.gz /data/kaspad

# Encrypt
tar czf - $BACKUP_DIR | gpg --encrypt --recipient your@email.com > $BACKUP_DIR.tar.gz.gpg
rm -rf $BACKUP_DIR

echo "Backup complete: $BACKUP_DIR.tar.gz.gpg"
```

## Troubleshooting

### Service won't start

```bash
# Check logs
docker-compose logs kaspa-rpc-service

# Common issues:
# 1. Kaspa node not ready → Wait for sync
# 2. Port 8080 in use → Change BIND_ADDRESS
# 3. Permission denied → Check docker socket access
```

### High memory usage

```bash
# Limit container memory
docker-compose up -d --scale kaspa-rpc-service=1 \
  --memory="1g" --memory-swap="1g"
```

### Slow performance

```bash
# Check Kaspa node performance
docker-compose exec kaspad kaspactl get-info

# Check network latency
docker-compose exec kaspa-rpc-service ping kaspad

# View metrics
curl http://localhost:8080/metrics | grep latency
```

## Security Checklist

- [ ] Change default JWT_SECRET
- [ ] Enable HTTPS (reverse proxy)
- [ ] Firewall rules (block direct Kaspa node access)
- [ ] Regular updates (Kaspa node + service)
- [ ] Monitor logs for suspicious activity
- [ ] Implement rate limiting
- [ ] Use strong authentication
- [ ] Regular security audits

## Scaling Guidelines

| Load | Setup | Resources |
|------|-------|-----------|
| < 100 req/s | Single instance | 1 CPU, 512MB RAM |
| 100-1000 req/s | 3 instances + load balancer | 2 CPU, 1GB RAM each |
| > 1000 req/s | Kubernetes cluster | Auto-scale 5-20 pods |

## Maintenance

### Updates

```bash
# Pull latest image
docker-compose pull kaspa-rpc-service

# Restart with zero downtime (if load balanced)
docker-compose up -d --no-deps --scale kaspa-rpc-service=2 kaspa-rpc-service
# Wait 30s
docker-compose up -d --no-deps --scale kaspa-rpc-service=1 kaspa-rpc-service
```

### Log Rotation

```bash
# Add to /etc/logrotate.d/kaspa-rpc
/var/log/kaspa-rpc/*.log {
    daily
    rotate 7
    compress
    delaycompress
    notifempty
    create 0640 kaspa kaspa
    postrotate
        systemctl reload kaspa-rpc
    endscript
}
```

## Cost Estimates

### Cloud Hosting (Monthly)

| Provider | Instance Type | Cost | Notes |
|----------|--------------|------|-------|
| AWS | t3.small | $15 | + data transfer |
| GCP | e2-small | $13 | + network egress |
| DigitalOcean | Basic Droplet 2GB | $18 | Flat rate |
| Hetzner | CX21 | €5 | Best value |

Add:
- Bandwidth: $5-50/month depending on traffic
- Storage: $1-5/month
- Load balancer: $10-20/month (if needed)

**Total**: $30-100/month for small deployment

## Support

- Technical issues: See TECHNICAL-SPEC.md
- Kaspa node issues: https://discord.gg/kaspa
- Security concerns: Open GitHub security advisory
