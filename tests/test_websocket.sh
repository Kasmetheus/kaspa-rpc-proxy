#!/bin/bash
# Test WebSocket subscribeUTXO endpoint

set -e

BASE_URL="${WS_URL:-ws://localhost:8080}"

echo "Testing WebSocket UTXO subscription..."
echo "(This will run for 30 seconds listening for UTXO updates)"

# Example addresses (replace with actual testnet addresses)
ADDRESSES="kaspa:qztest1234567890abcdef,kaspa:qztest0987654321fedcba"

# Use websocat if available, otherwise provide instructions
if command -v websocat &> /dev/null; then
  timeout 30 websocat "${BASE_URL}/ws/subscribeUTXO?addresses=${ADDRESSES}" || true
else
  echo "⚠️  websocat not found. Install with: cargo install websocat"
  echo ""
  echo "Test manually with:"
  echo "  websocat '${BASE_URL}/ws/subscribeUTXO?addresses=${ADDRESSES}'"
  echo ""
  echo "Or use this JavaScript snippet in browser console:"
  echo ""
  cat <<'EOF'
const ws = new WebSocket('ws://localhost:8080/ws/subscribeUTXO?addresses=kaspa:qz...');
ws.onopen = () => console.log('✅ Connected');
ws.onmessage = (e) => console.log('📨', JSON.parse(e.data));
ws.onerror = (e) => console.error('❌', e);
ws.onclose = () => console.log('Closed');
EOF
fi
