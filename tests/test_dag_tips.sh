#!/bin/bash
# Test getDAGTips endpoint

set -e

BASE_URL="${BASE_URL:-http://localhost:8080}"

echo "Testing getDAGTips endpoint..."

response=$(curl -s -X POST "${BASE_URL}/rpc/getDAGTips" \
  -H "Content-Type: application/json" \
  -d "{}")

echo "Response:"
echo "$response" | jq '.'

# Check if successful
if echo "$response" | jq -e '.success == true' > /dev/null; then
  echo "✅ Test passed"
  latency=$(echo "$response" | jq -r '.latency_ms')
  echo "⏱️  Latency: ${latency}ms"
  
  # Extract some interesting data
  block_count=$(echo "$response" | jq -r '.data.blockCount')
  virtual_daa=$(echo "$response" | jq -r '.data.virtualDaaScore')
  tip_count=$(echo "$response" | jq -r '.data.tipHashes | length')
  
  echo "📊 Block count: ${block_count}"
  echo "📊 Virtual DAA score: ${virtual_daa}"
  echo "📊 Tip hashes: ${tip_count}"
  
  if (( $(echo "$latency < 50" | bc -l) )); then
    echo "✅ Latency under 50ms target"
  else
    echo "⚠️  Latency exceeds 50ms target"
  fi
else
  echo "❌ Test failed"
  exit 1
fi
