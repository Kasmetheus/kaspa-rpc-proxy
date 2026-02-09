# Kaspa RPC Service

High-performance, production-ready RPC proxy service for Kaspa blockchain. Built with Rust for maximum reliability and minimal latency.

## 🎯 Features

- **4 Core Endpoints**: getBlock, submitTransaction, subscribeUTXO, getDAGTips
- **WebSocket Support**: Real-time UTXO change notifications
- **Sub-50ms Latency**: Optimized for performance with built-in metrics
- **JWT Authentication**: Security skeleton ready for production
- **Docker Deployment**: Testnet-ready with docker-compose
- **99.999% Uptime Design**: Health checks, auto-restart, error recovery
- **Self-Contained**: No external dependencies beyond Kaspa node

## 📋 Requirements

- Rust 1.85+ (for local development)
- Docker & Docker Compose (for deployment)
- Kaspa testnet/mainnet node with gRPC enabled

## 🚀 Quick Start

### Using Docker (Recommended)

1. **Clone and configure**:
   ```bash
   cd kaspa-rpc-service
   cp .env.example .env
   # Edit .env with your JWT secret
   ```

2. **Start services** (includes Kaspa testnet node):
   ```bash
   docker-compose up -d
   ```

3. **Check health**:
   ```bash
   curl http://localhost:8080/health
   ```

### Local Development

1. **Install dependencies**:
   ```bash
   # Ensure protobuf compiler is installed
   # macOS:
   brew install protobuf
   # Ubuntu/Debian:
   sudo apt install protobuf-compiler
   ```

2. **Build**:
   ```bash
   cargo build --release
   ```

3. **Run**:
   ```bash
   # Ensure Kaspa node is running on localhost:16110
   KASPA_RPC_URL=http://localhost:16110 cargo run --release
   ```

## 📡 API Endpoints

### HTTP Endpoints

All endpoints accept JSON POST requests and return JSON responses with latency metrics.

#### 1. Get Block

**Endpoint**: `POST /rpc/getBlock`

**Request**:
```json
{
  "hash": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "includeTransactions": true
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "hash": "0123...",
    "header": {
      "version": 1,
      "timestamp": 1707523200,
      "daaScore": 12345678,
      "blueScore": 12345677,
      ...
    },
    "transactions": [...]
  },
  "latency_ms": 12.5
}
```

#### 2. Submit Transaction

**Endpoint**: `POST /rpc/submitTransaction`

**Request**:
```json
{
  "transaction": {
    "version": 0,
    "inputs": [
      {
        "previousOutpoint": {
          "transactionId": "abc123...",
          "index": 0
        },
        "signatureScript": "deadbeef...",
        "sequence": 0
      }
    ],
    "outputs": [
      {
        "amount": 100000000,
        "scriptPublicKey": {
          "scriptPublicKey": "76a914...",
          "version": 0
        }
      }
    ]
  },
  "allowOrphan": false
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "transactionId": "def456..."
  },
  "latency_ms": 8.3
}
```

#### 3. Get DAG Tips

**Endpoint**: `POST /rpc/getDAGTips`

**Request**:
```json
{}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "tipHashes": ["abc...", "def..."],
    "blockCount": 12345678,
    "headerCount": 12345680,
    "difficulty": 1234567.89,
    "virtualDaaScore": 12345677,
    "virtualParentHashes": ["ghi..."],
    "pruningPointHash": "jkl..."
  },
  "latency_ms": 5.1
}
```

### WebSocket Endpoint

#### 4. Subscribe to UTXO Changes

**Endpoint**: `GET /ws/subscribeUTXO?addresses=kaspa:qz....,kaspa:qp....`

**Connection**:
```javascript
const ws = new WebSocket('ws://localhost:8080/ws/subscribeUTXO?addresses=kaspa:qz1234...,kaspa:qp5678...');

ws.onopen = () => console.log('Connected');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('UTXO Update:', data);
};
```

**Messages**:
```json
{
  "type": "utxo_changed",
  "added": [
    {
      "address": "kaspa:qz...",
      "outpoint": {
        "transaction_id": "abc...",
        "index": 0
      },
      "utxo_entry": {
        "amount": 100000000,
        "blockDaaScore": 12345678,
        "isCoinbase": false
      }
    }
  ],
  "removed": [...]
}
```

## 🔒 Authentication

JWT authentication skeleton is included but not enforced by default.

### Generate Token (for future use):
```rust
use kaspa_rpc_service::auth;

let token = auth::generate_token("user123", "your-secret", "user")?;
```

### Use Token:
```bash
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -X POST http://localhost:8080/rpc/getDAGTips
```

## 📊 Monitoring

### Metrics Endpoint

**Endpoint**: `GET /metrics`

Returns Prometheus-compatible metrics including:
- Request latency histograms per endpoint
- Request counts
- Error rates

**Example**:
```
# HELP kaspa_rpc_get_block_latency_ms Latency for get_block endpoint in milliseconds
# TYPE kaspa_rpc_get_block_latency_ms histogram
kaspa_rpc_get_block_latency_ms_bucket{le="1"} 0
kaspa_rpc_get_block_latency_ms_bucket{le="5"} 23
kaspa_rpc_get_block_latency_ms_bucket{le="10"} 145
kaspa_rpc_get_block_latency_ms_bucket{le="25"} 203
kaspa_rpc_get_block_latency_ms_bucket{le="50"} 205
...
```

### Latency Targets

- **Target**: < 50ms for all operations
- **Design**: 99.9% of requests under 50ms
- **Warning**: Logs generated when latency exceeds 50ms

## 🧪 Testing

### Manual Testing

Test scripts are provided in `tests/`:

```bash
# Test getBlock
./tests/test_get_block.sh

# Test submitTransaction (requires valid tx)
./tests/test_submit_tx.sh

# Test getDAGTips
./tests/test_dag_tips.sh

# Test WebSocket subscription
./tests/test_websocket.sh
```

### Load Testing

```bash
# Install wrk
brew install wrk  # macOS
apt install wrk   # Ubuntu

# Run load test
wrk -t4 -c100 -d30s --latency \
  -s tests/wrk_test.lua \
  http://localhost:8080
```

## 🏗️ Architecture

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │ HTTP/WS
       ▼
┌─────────────────────┐
│  Kaspa RPC Service  │
│  (Rust + Axum)      │
│  ┌──────────────┐   │
│  │  Handlers    │   │
│  │  Auth        │   │
│  │  Metrics     │   │
│  │  WebSocket   │   │
│  └──────┬───────┘   │
└─────────┼───────────┘
          │ gRPC
          ▼
┌─────────────────────┐
│   Kaspa Node        │
│   (rusty-kaspa)     │
└─────────────────────┘
```

## 🔧 Configuration

Environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `KASPA_RPC_URL` | `http://localhost:16110` | Kaspa node gRPC endpoint |
| `BIND_ADDRESS` | `0.0.0.0:8080` | Service bind address |
| `JWT_SECRET` | `CHANGE_ME_IN_PRODUCTION` | JWT signing secret |
| `RUST_LOG` | `kaspa_rpc_service=debug` | Logging level |

## 🛡️ Production Hardening

### Security Checklist

- [ ] Change `JWT_SECRET` to strong random value (32+ chars)
- [ ] Enable JWT authentication on all endpoints
- [ ] Use HTTPS/WSS in production (reverse proxy)
- [ ] Implement rate limiting
- [ ] Add IP whitelisting if needed
- [ ] Review and restrict CORS policy

### Reliability Checklist

- [ ] Set up Prometheus monitoring
- [ ] Configure alerting for latency > 50ms
- [ ] Set up log aggregation
- [ ] Enable automatic restarts (systemd/docker)
- [ ] Configure backup Kaspa node endpoints
- [ ] Test failover scenarios

### Performance Tuning

```bash
# Increase file descriptor limits
ulimit -n 65535

# Optimize Rust release build
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Run with more worker threads
TOKIO_WORKER_THREADS=8 ./kaspa-rpc-service
```

## 📚 Technical Details

### Protocol Buffer Integration

Service uses Kaspa's native protobuf definitions from `rusty-kaspa/rpc/grpc/core/proto/`:
- `rpc.proto`: Core RPC types
- `messages.proto`: Request/response envelopes

Generated at build time via `build.rs`.

### Latency Optimization

1. **Connection Pooling**: gRPC channel reuse
2. **Zero-Copy**: Minimal serialization overhead
3. **Async I/O**: Tokio runtime for max concurrency
4. **Release Optimizations**: LTO, codegen-units=1

### Error Handling

- **Connection errors**: Automatic retry with backoff
- **Node errors**: Propagated with context
- **Invalid requests**: 400 Bad Request with details
- **Internal errors**: 500 with request ID for debugging

## 🐛 Troubleshooting

### Service won't start

```bash
# Check Kaspa node is reachable
grpcurl -plaintext localhost:16110 list

# Check logs
docker-compose logs kaspa-rpc-service

# Verify environment
docker-compose exec kaspa-rpc-service env
```

### High latency

```bash
# Check metrics
curl http://localhost:8080/metrics | grep latency

# Monitor Kaspa node performance
docker-compose logs kaspad | grep -i slow

# Check network connectivity
ping kaspad
```

### WebSocket disconnects

- Client must implement reconnection logic
- Check for firewall/proxy timeout settings
- Monitor for Kaspa node subscriptions limit

## 📝 License

MIT License - See LICENSE file

## 🤝 Contributing

This is a reference implementation. For production use:
1. Review all TODOs in source code
2. Complete JWT auth implementation
3. Add comprehensive tests
4. Configure monitoring and alerting

## 📞 Support

For Kaspa protocol questions: https://discord.gg/kaspa
For issues: Open a GitHub issue
