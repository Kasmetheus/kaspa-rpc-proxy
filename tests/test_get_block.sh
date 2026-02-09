#!/bin/bash
# Test getBlock endpoint

set -e

BASE_URL="${BASE_URL:-http://localhost:8080}"

echo "Testing getBlock endpoint..."

# Example block hash (replace with actual testnet block hash)
BLOCK_HASH="0000000000000000000000000000000000000000000000000000000000000001"

response=$(curl -s -X POST "${BASE_URL}/rpc/getBlock" \
  -H "Content-Type: application/json" \
  -d "{
    \"hash\": \"${BLOCK_HASH}\",
    \"includeTransactions\": true
  }")

echo "Response:"
echo "$response" | jq '.'

# Check if successful
if echo "$response" | jq -e '.success == true' > /dev/null; then
  echo "✅ Test passed"
  latency=$(echo "$response" | jq -r '.latency_ms')
  echo "⏱️  Latency: ${latency}ms"
  
  if (( $(echo "$latency < 50" | bc -l) )); then
    echo "✅ Latency under 50ms target"
  else
    echo "⚠️  Latency exceeds 50ms target"
  fi
else
  echo "❌ Test failed"
  exit 1
fi
