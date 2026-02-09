#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"
LOG="../kaspa-rpc-deploy-log.md"
echo -e "\n## Monitoring & Deploy Start: $(date)" | tee -a "$LOG"
# Start deploy if not running
if ! docker compose ps | grep -q Up; then
  echo "No services up, starting docker compose up -d" | tee -a "$LOG"
  docker compose up -d | tee -a "$LOG"
fi
START_TIME=$(date +%s)
STUCK_THRESHOLD=600  # 10 min
MAX_RUNTIME=1800  # 30 min total
while true; do
  CURRENT_TIME=$(date +%s)
  ELAPSED=$((CURRENT_TIME - START_TIME))
  if [ $ELAPSED -gt $MAX_RUNTIME ]; then
    echo "Max runtime exceeded, stopping." | tee -a "$LOG"
    docker compose down
    exit 1
  fi
  echo -e "\n### Poll at $(date) (elapsed: ${ELAPSED}s)" | tee -a "$LOG"
  # Recent logs
  docker compose logs --tail=20 | tee -a "$LOG"
  # Health
  KASPAD_HEALTH=$(docker inspect kaspad-testnet --format='{{.State.Health.Status}}' 2>/dev/null || echo "N/A")
  RPC_HEALTH=$(docker inspect kaspa-rpc-service --format='{{.State.Health.Status}}' 2>/dev/null || echo "N/A")
  echo "kaspad health: $KASPAD_HEALTH" | tee -a "$LOG"
  echo "rpc health: $RPC_HEALTH" | tee -a "$LOG"
  # RPC endpoint test
  RPC_TEST=""
  if docker ps | grep -q kaspa-rpc-service; then
    RPC_TEST=$(curl -s -f -m 5 http://localhost:8080/health || echo "FAIL")
  fi
  echo "RPC /health: $RPC_TEST" | tee -a "$LOG"
  # Check if up
  if [ "$KASPAD_HEALTH" = "healthy" ] && [ "$RPC_HEALTH" = "healthy" ] && [ "$RPC_TEST" != "FAIL" ]; then
    echo -e "\n## 🎉 KASPAD/RPC UP AND HEALTHY! $(date)" | tee -a "$LOG"
    # Test load: JSON-RPC calls
    BLOCK_COUNT=$(curl -s -m 5 -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"blockCount","params":{},"id":1}' http://localhost:8080/ 2>/dev/null || echo "RPC call fail")
    echo "Block count test: $BLOCK_COUNT" | tee -a "$LOG"
    INFO=$(curl -s -m 5 http://localhost:8080/info 2>/dev/null || echo "no info")
    echo "Info endpoint: $INFO" | tee -a "$LOG"
    echo "## Monitoring complete: Services are up and tested." | tee -a "$LOG"
    exit 0
  fi
  # Check stuck
  if [ $ELAPSED -gt $STUCK_THRESHOLD ] && { [ "$KASPAD_HEALTH" != "healthy" ] || [ "$RPC_HEALTH" != "healthy" ]; }; then
    echo -e "\n## 🔄 STUCK >10min, killing and restarting: $(date)" | tee -a "$LOG"
    docker compose down | tee -a "$LOG"
    sleep 10
    docker compose up -d --force-recreate --build | tee -a "$LOG"
    START_TIME=$(date +%s)
    echo "Restart complete." | tee -a "$LOG"
  fi
  sleep 30
done