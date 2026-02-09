#!/bin/bash
# Health and metrics check

set -e

BASE_URL="${BASE_URL:-http://localhost:8080}"

echo "Checking service health..."

# Health endpoint
health=$(curl -s "${BASE_URL}/health" -o /dev/null -w "%{http_code}")

if [ "$health" = "200" ]; then
  echo "✅ Health check passed (HTTP $health)"
else
  echo "❌ Health check failed (HTTP $health)"
  exit 1
fi

# Metrics endpoint
echo ""
echo "Fetching metrics..."
metrics=$(curl -s "${BASE_URL}/metrics")

if [ -n "$metrics" ]; then
  echo "✅ Metrics available"
  echo ""
  echo "Sample metrics:"
  echo "$metrics" | grep -E "kaspa_rpc.*latency" | head -10
else
  echo "❌ No metrics found"
  exit 1
fi

echo ""
echo "🎉 All checks passed!"
