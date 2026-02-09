# Kaspa RPC Service - Technical Specification

## Overview

High-performance RPC proxy service providing a simplified JSON/HTTP and WebSocket interface to Kaspa's native gRPC API.

## Architecture

### Stack

- **Language**: Rust 1.85+
- **Web Framework**: Axum 0.7 (built on Tokio async runtime)
- **Protocol**: gRPC (client to Kaspa), HTTP/WebSocket (client-facing)
- **Serialization**: Protobuf (Kaspa), JSON (HTTP API)
- **Authentication**: JWT (jsonwebtoken crate)
- **Metrics**: Prometheus

### Design Principles

1. **Zero External Dependencies**: Self-contained service, no third-party APIs
2. **Latency First**: Sub-50ms response time target
3. **Reliability**: 99.999% uptime design with graceful degradation
4. **Accuracy**: 99.9% data fidelity from Kaspa node
5. **Security**: JWT auth ready, no secrets in logs

## Component Details

### 1. gRPC Client (`src/client.rs`)

**Purpose**: Manage persistent gRPC connection to Kaspa node

**Key Features**:
- Connection pooling via tonic::Channel
- Automatic reconnection on disconnect
- Request ID generation for tracing
- Type-safe proto message handling

**Methods**:
- `get_block()` - Fetch block by hash
- `submit_transaction()` - Broadcast transaction
- `get_dag_tips()` - Get DAG state
- `subscribe_utxo_changes()` - Stream UTXO updates

**Performance**:
- Channel reuse eliminates connection overhead
- Zero-copy where possible
- Async streaming for subscriptions

### 2. HTTP Handlers (`src/handlers.rs`)

**Purpose**: Translate HTTP requests to gRPC calls

**Endpoints**:

```
POST /rpc/getBlock
POST /rpc/submitTransaction  
POST /rpc/getDAGTips
GET  /health
GET  /metrics
```

**Latency Tracking**:
- Start timer on request entry
- Record to Prometheus histogram on completion
- Include in JSON response
- Log warning if > 50ms

**Error Handling**:
- Kaspa errors → 400 Bad Request
- Connection errors → 502 Bad Gateway
- Invalid input → 400 Bad Request with details
- Internal errors → 500 with request ID

### 3. WebSocket Server (`src/websocket.rs`)

**Purpose**: Real-time UTXO change notifications

**Protocol**:
1. Client connects with addresses in query params
2. Service subscribes to Kaspa gRPC stream
3. Forward notifications as JSON messages
4. Auto-reconnect on server disconnect

**Message Format**:
```json
{
  "type": "utxo_changed",
  "added": [...],
  "removed": [...]
}
```

**Scalability**:
- One gRPC subscription per WebSocket client
- Async message forwarding
- Backpressure handling

### 4. Authentication (`src/auth.rs`)

**JWT Structure**:
```json
{
  "sub": "user_id",
  "exp": 1707523200,
  "iat": 1707519600,
  "role": "admin|user"
}
```

**Implementation Status**: Skeleton only
**Production TODO**:
- Implement middleware extractor
- Add role-based access control
- Token refresh mechanism
- Revocation list

### 5. Metrics (`src/metrics.rs`)

**Prometheus Metrics**:
- `kaspa_rpc_{endpoint}_latency_ms` - Histogram (1-1000ms buckets)
- Request counts (implicit)
- Error rates (implicit)

**Monitoring Strategy**:
- Alert on p99 latency > 50ms
- Alert on error rate > 0.1%
- Track WebSocket connection count

## Data Flow

### Synchronous Request (getBlock)

```
┌──────┐      ┌─────────┐      ┌────────┐
│Client│─────▶│Handler  │─────▶│gRPC    │
│      │ HTTP │ (JSON)  │ gRPC │Client  │
└──────┘      └─────────┘      └────────┘
   ▲                                │
   │         ┌─────────┐           │
   └─────────│Response │◀──────────┘
             │+ Latency│
             └─────────┘
```

### Asynchronous Stream (subscribeUTXO)

```
┌──────┐      ┌──────────┐      ┌────────┐
│Client│◀═════│WebSocket │◀═════│gRPC    │
│      │  WS  │Handler   │ gRPC │Stream  │
└──────┘      └──────────┘      └────────┘
              Persistent         Persistent
              Connection         Subscription
```

## Protocol Buffer Schema

Source: `rusty-kaspa/rpc/grpc/core/proto/`

**Key Messages**:
- `KaspadRequest`/`KaspadResponse` - Top-level envelope
- `GetBlockRequestMessage` → `GetBlockResponseMessage`
- `SubmitTransactionRequestMessage` → `SubmitTransactionResponseMessage`
- `GetBlockDagInfoRequestMessage` → `GetBlockDagInfoResponseMessage`
- `NotifyUtxosChangedRequestMessage` + `UtxosChangedNotificationMessage`

**Build Process**:
1. `build.rs` runs `tonic-build`
2. Generates Rust types from `.proto` files
3. Includes in `src/client.rs` via `tonic::include_proto!`

## Performance Optimization

### Compiler Flags (Cargo.toml)

```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization, slower compile
strip = true         # Remove debug symbols
```

### Runtime Tuning

- **Tokio Workers**: Auto-detect CPU cores
- **gRPC Channels**: Single shared channel per endpoint
- **Buffer Sizes**: 8KB default, tune per deployment

### Benchmarks (Expected)

| Endpoint | p50 | p95 | p99 | Target |
|----------|-----|-----|-----|--------|
| getBlock | 8ms | 15ms | 25ms | <50ms |
| submitTx | 12ms | 20ms | 35ms | <50ms |
| getDAGTips | 3ms | 8ms | 15ms | <50ms |

## Reliability Design

### Failure Modes

1. **Kaspa Node Down**:
   - Return 502 Bad Gateway
   - Log error
   - TODO: Implement failover to backup node

2. **gRPC Stream Disconnect**:
   - Auto-reconnect with exponential backoff
   - Notify WebSocket clients of reconnection

3. **High Load**:
   - Tokio task queue handles backpressure
   - TODO: Implement rate limiting

### Health Checks

- **Service**: `/health` returns 200 if running
- **Kaspa Node**: Verify gRPC connectivity on startup
- **Docker**: Health check every 30s with 3 retries

### Logging

Levels:
- `ERROR`: Critical failures requiring attention
- `WARN`: Latency > 50ms, connection issues
- `INFO`: Startup, shutdown, subscriptions
- `DEBUG`: Request/response details

Format: JSON structured logs (TODO)

## Security Considerations

### Current State (Week 1 Prototype)

✅ JWT skeleton implemented
✅ No secrets in logs
✅ Input validation on hashes
❌ Authentication not enforced
❌ No rate limiting
❌ No TLS (use reverse proxy)

### Production Hardening TODO

1. **Authentication**:
   - Enforce JWT on all endpoints except /health
   - Implement token refresh
   - Add revocation mechanism

2. **Authorization**:
   - Role-based access (admin vs user)
   - Per-address UTXO subscription limits

3. **Rate Limiting**:
   - Per-IP: 100 req/min
   - Per-user: 1000 req/min
   - WebSocket: Max 10 concurrent connections

4. **Input Validation**:
   - Max transaction size
   - Address format validation
   - Request size limits

5. **Network**:
   - TLS termination via reverse proxy
   - Firewall whitelist for Kaspa node connection
   - DDoS protection (Cloudflare/AWS Shield)

## Deployment

### Docker Multi-Stage Build

Stage 1 (builder):
- Install build deps (protobuf, etc.)
- Cargo build --release
- Size: ~2GB

Stage 2 (runtime):
- Minimal Debian base
- Copy binary only
- Size: ~50MB

### Resource Requirements

| Environment | CPU | RAM | Disk | Network |
|-------------|-----|-----|------|---------|
| Testnet | 1 core | 512MB | 1GB | 10 Mbps |
| Mainnet | 2 cores | 1GB | 5GB | 100 Mbps |

### Scaling

- **Vertical**: Increase CPU for more Tokio workers
- **Horizontal**: Load balance multiple instances (stateless)
- **Kaspa Node**: Dedicate nodes per RPC service instance

## Testing Strategy

### Unit Tests (TODO)

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_get_block_valid_hash() { ... }
    
    #[tokio::test]
    async fn test_submit_transaction_invalid() { ... }
}
```

### Integration Tests

Provided shell scripts:
- `tests/test_get_block.sh`
- `tests/test_dag_tips.sh`
- `tests/test_websocket.sh`
- `tests/health_check.sh`

### Load Testing

Tool: `wrk` or `k6`

Target:
- 1000 concurrent connections
- 10,000 requests/second
- < 50ms p99 latency maintained

## Monitoring & Observability

### Metrics (Prometheus)

```
# Latency histograms
kaspa_rpc_get_block_latency_ms{le="50"} 203

# Request counts
kaspa_rpc_get_block_latency_ms_count 205

# Error tracking (TODO)
kaspa_rpc_errors_total{endpoint="getBlock", type="kaspa_error"} 2
```

### Grafana Dashboard (TODO)

Panels:
- Latency over time (per endpoint)
- Request rate
- Error rate
- Active WebSocket connections
- Kaspa node health

### Alerting Rules

```yaml
- alert: HighLatency
  expr: kaspa_rpc_latency_ms{quantile="0.99"} > 50
  for: 5m
  
- alert: HighErrorRate
  expr: rate(kaspa_rpc_errors_total[5m]) > 0.01
  for: 2m
```

## Future Enhancements

### Phase 2 (Week 2+)

1. **Caching Layer**:
   - Redis for block cache (1000 recent blocks)
   - TTL: 10 seconds for DAG tips
   - Cache hit rate target: >80%

2. **Request Batching**:
   - Batch multiple getBlock requests
   - Reduce gRPC overhead

3. **GraphQL API**:
   - Alternative to REST
   - Flexible queries

4. **SDK Generation**:
   - OpenAPI spec
   - Auto-generate client libraries

### Phase 3 (Production)

1. **Multi-Node Support**:
   - Health check Kaspa nodes
   - Automatic failover
   - Load balancing

2. **Advanced Monitoring**:
   - Distributed tracing (Jaeger)
   - Request ID propagation
   - Latency breakdown

3. **Compliance**:
   - Audit logging
   - GDPR considerations (address privacy)

## References

**Kaspa Protocol**:
- Protobuf definitions: `rusty-kaspa/rpc/grpc/core/proto/`
- RPC docs: https://github.com/kaspanet/rusty-kaspa/tree/master/rpc
- Network protocol: https://kaspa.org/protocol

**Dependencies**:
- Tokio: https://tokio.rs/
- Axum: https://github.com/tokio-rs/axum
- Tonic: https://github.com/hyperium/tonic
- Prometheus: https://prometheus.io/

---

**Document Version**: 1.0  
**Last Updated**: 2026-02-09  
**Status**: Week 1 Prototype Complete
