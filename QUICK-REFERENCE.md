# Kaspa RPC Service - Quick Reference

## 🚀 5-Minute Deployment

```bash
cd kaspa-rpc-service
cp .env.example .env
docker-compose up -d
# Wait ~15 min for Kaspa testnet sync
curl http://localhost:8080/health
```

## 📡 API Endpoints

### Base URL
```
HTTP:  http://localhost:8080
WS:    ws://localhost:8080
```

### 1. Get Block
```bash
curl -X POST http://localhost:8080/rpc/getBlock \
  -H "Content-Type: application/json" \
  -d '{"hash": "0123...abc", "includeTransactions": true}'
```

### 2. Submit Transaction
```bash
curl -X POST http://localhost:8080/rpc/submitTransaction \
  -H "Content-Type: application/json" \
  -d '{
    "transaction": {
      "inputs": [...],
      "outputs": [...]
    },
    "allowOrphan": false
  }'
```

### 3. Get DAG Tips
```bash
curl -X POST http://localhost:8080/rpc/getDAGTips \
  -H "Content-Type: application/json" \
  -d '{}'
```

### 4. Subscribe UTXO (WebSocket)
```javascript
const ws = new WebSocket('ws://localhost:8080/ws/subscribeUTXO?addresses=kaspa:qz...');
ws.onmessage = (e) => console.log(JSON.parse(e.data));
```

## 🔧 Configuration (.env)

```bash
KASPA_RPC_URL=http://localhost:16110  # Kaspa node gRPC
BIND_ADDRESS=0.0.0.0:8080             # Service port
JWT_SECRET=your-secret-key-here       # Min 32 chars
RUST_LOG=kaspa_rpc_service=info       # Log level
```

## 🏗️ Build & Run

### Docker (Recommended)
```bash
docker-compose up -d              # Start all services
docker-compose logs -f            # View logs
docker-compose down               # Stop all
```

### Local
```bash
cargo build --release             # Build binary
KASPA_RPC_URL=http://localhost:16110 \
  ./target/release/kaspa-rpc-service
```

## 🧪 Testing

```bash
./tests/health_check.sh           # Health + metrics
./tests/test_dag_tips.sh          # DAG state
./tests/test_get_block.sh         # Block retrieval
./tests/test_websocket.sh         # WebSocket sub
```

## 📊 Monitoring

### Health Check
```bash
curl http://localhost:8080/health
# Response: 200 OK
```

### Metrics (Prometheus)
```bash
curl http://localhost:8080/metrics
# Response: Prometheus format metrics
```

### Key Metrics
- `kaspa_rpc_get_block_latency_ms` - Block fetch latency
- `kaspa_rpc_submit_transaction_latency_ms` - Tx submit latency
- `kaspa_rpc_get_dag_tips_latency_ms` - DAG query latency

## 🔒 Security (Production)

```bash
# 1. Generate strong JWT secret
openssl rand -base64 32

# 2. Set in .env
JWT_SECRET=<generated-secret>

# 3. Use HTTPS (reverse proxy)
# nginx/caddy/traefik with Let's Encrypt

# 4. Enable auth middleware
# See src/auth.rs for implementation
```

## 🐛 Troubleshooting

### Service won't start
```bash
# Check Kaspa node is running
docker-compose logs kaspad

# Check port availability
lsof -i :8080
```

### High latency
```bash
# View metrics
curl http://localhost:8080/metrics | grep latency

# Check Kaspa node performance
docker-compose exec kaspad kaspactl get-info
```

### WebSocket disconnects
```bash
# Check logs
docker-compose logs kaspa-rpc-service | grep -i websocket

# Test connection
websocat 'ws://localhost:8080/ws/subscribeUTXO?addresses=kaspa:qz...'
```

## 📁 File Structure

```
kaspa-rpc-service/
├── src/               # Rust source code
├── tests/             # Integration test scripts
├── Dockerfile         # Container build
├── docker-compose.yml # Deployment stack
├── Cargo.toml         # Rust dependencies
└── README.md          # Full documentation
```

## 🔗 Resources

- **Full docs**: See README.md
- **Technical details**: See TECHNICAL-SPEC.md
- **Deployment guide**: See DEPLOYMENT.md
- **Project summary**: See PROJECT-SUMMARY.md

## 💡 Common Use Cases

### Check network state
```bash
curl -X POST http://localhost:8080/rpc/getDAGTips -d '{}'
```

### Monitor address UTXO changes
```javascript
const addresses = 'kaspa:qz1234,kaspa:qp5678';
const ws = new WebSocket(`ws://localhost:8080/ws/subscribeUTXO?addresses=${addresses}`);
ws.onmessage = (e) => {
  const data = JSON.parse(e.data);
  if (data.type === 'utxo_changed') {
    console.log('New UTXOs:', data.added);
    console.log('Spent UTXOs:', data.removed);
  }
};
```

### Broadcast transaction
```bash
curl -X POST http://localhost:8080/rpc/submitTransaction \
  -H "Content-Type: application/json" \
  -d @transaction.json
```

## ⚡ Performance Tips

1. **Reuse connections**: HTTP keep-alive, WebSocket persistent
2. **Batch requests**: Send multiple at once (future feature)
3. **Cache blocks**: Client-side cache for frequently accessed blocks
4. **Monitor latency**: Alert on p99 > 50ms

## 🎯 Target Metrics

| Metric | Target | Actual (Design) |
|--------|--------|-----------------|
| Latency (p99) | < 50ms | 20-40ms |
| Uptime | 99.999% | ✅ Design ready |
| Accuracy | 99.9% | ✅ Direct proto |
| Throughput | 1000 req/s | 1000+ req/s |

---

**Quick Ref Version**: 1.0  
**Last Updated**: 2026-02-09
