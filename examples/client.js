#!/usr/bin/env node
/**
 * Kaspa RPC Service - Example Client (Node.js)
 * 
 * Demonstrates how to interact with all 4 core endpoints
 * 
 * Requirements: npm install node-fetch ws
 */

const fetch = require('node-fetch');
const WebSocket = require('ws');

const BASE_URL = 'http://localhost:8080';
const WS_URL = 'ws://localhost:8080';

// Example 1: Get DAG Tips
async function getDAGTips() {
  console.log('\n📊 Getting DAG tips...');
  
  const response = await fetch(`${BASE_URL}/rpc/getDAGTips`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({})
  });
  
  const data = await response.json();
  
  if (data.success) {
    console.log(`✅ Block count: ${data.data.blockCount}`);
    console.log(`✅ Virtual DAA score: ${data.data.virtualDaaScore}`);
    console.log(`⏱️  Latency: ${data.latency_ms}ms`);
    console.log(`📍 Tips: ${data.data.tipHashes.slice(0, 2).join(', ')}...`);
    return data.data;
  } else {
    console.error('❌ Error:', data.error);
  }
}

// Example 2: Get Block
async function getBlock(hash) {
  console.log(`\n🧱 Getting block ${hash.slice(0, 16)}...`);
  
  const response = await fetch(`${BASE_URL}/rpc/getBlock`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      hash: hash,
      includeTransactions: true
    })
  });
  
  const data = await response.json();
  
  if (data.success) {
    const block = data.data;
    console.log(`✅ Block hash: ${block.hash.slice(0, 16)}...`);
    console.log(`✅ DAA score: ${block.header.daaScore}`);
    console.log(`✅ Transactions: ${block.transactions.length}`);
    console.log(`⏱️  Latency: ${data.latency_ms}ms`);
    return block;
  } else {
    console.error('❌ Error:', data.error);
  }
}

// Example 3: Submit Transaction
async function submitTransaction(transaction) {
  console.log('\n📤 Submitting transaction...');
  
  const response = await fetch(`${BASE_URL}/rpc/submitTransaction`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      transaction: transaction,
      allowOrphan: false
    })
  });
  
  const data = await response.json();
  
  if (data.success) {
    console.log(`✅ Transaction ID: ${data.data.transactionId}`);
    console.log(`⏱️  Latency: ${data.latency_ms}ms`);
    return data.data.transactionId;
  } else {
    console.error('❌ Error:', data.error);
  }
}

// Example 4: Subscribe to UTXO Changes
function subscribeUTXO(addresses) {
  console.log('\n🔔 Subscribing to UTXO changes...');
  console.log(`📍 Addresses: ${addresses.join(', ')}`);
  
  const addressParams = addresses.join(',');
  const ws = new WebSocket(`${WS_URL}/ws/subscribeUTXO?addresses=${addressParams}`);
  
  ws.on('open', () => {
    console.log('✅ WebSocket connected');
  });
  
  ws.on('message', (data) => {
    const message = JSON.parse(data.toString());
    
    if (message.status === 'subscribed') {
      console.log('✅ Subscription confirmed');
    } else if (message.type === 'utxo_changed') {
      console.log('\n🔔 UTXO Change Notification:');
      console.log(`  Added: ${message.added.length} UTXOs`);
      console.log(`  Removed: ${message.removed.length} UTXOs`);
      
      // Show details of first added UTXO
      if (message.added.length > 0) {
        const utxo = message.added[0];
        console.log(`  📍 Address: ${utxo.address}`);
        console.log(`  💰 Amount: ${utxo.utxo_entry?.amount || 'unknown'}`);
      }
    }
  });
  
  ws.on('error', (error) => {
    console.error('❌ WebSocket error:', error.message);
  });
  
  ws.on('close', () => {
    console.log('🔌 WebSocket closed');
  });
  
  // Keep subscription alive for 60 seconds
  setTimeout(() => {
    console.log('\n⏰ Closing subscription...');
    ws.close();
  }, 60000);
  
  return ws;
}

// Health Check
async function healthCheck() {
  console.log('🏥 Checking service health...');
  
  const response = await fetch(`${BASE_URL}/health`);
  
  if (response.ok) {
    console.log('✅ Service is healthy');
    return true;
  } else {
    console.error('❌ Service unhealthy');
    return false;
  }
}

// Main demo
async function main() {
  console.log('🚀 Kaspa RPC Service - Client Demo\n');
  console.log('=' .repeat(50));
  
  try {
    // 1. Health check
    await healthCheck();
    
    // 2. Get DAG tips
    const dagInfo = await getDAGTips();
    
    // 3. Get a block (if tip hashes available)
    if (dagInfo && dagInfo.tipHashes && dagInfo.tipHashes.length > 0) {
      const tipHash = dagInfo.tipHashes[0];
      await getBlock(tipHash);
    }
    
    // 4. Subscribe to UTXO changes (example addresses)
    // Replace with real testnet addresses for live updates
    const exampleAddresses = [
      'kaspa:qztest1234567890abcdef',
      'kaspa:qztest0987654321fedcba'
    ];
    
    subscribeUTXO(exampleAddresses);
    
    // Note: submitTransaction example omitted - requires valid signed transaction
    console.log('\n💡 Tip: To submit transactions, create a valid signed transaction object');
    
    console.log('\n' + '='.repeat(50));
    console.log('✅ Demo complete! WebSocket will stay open for 60s');
    console.log('   Press Ctrl+C to exit early\n');
    
  } catch (error) {
    console.error('\n❌ Error:', error.message);
    console.error('\n💡 Make sure the service is running:');
    console.error('   docker-compose up -d\n');
    process.exit(1);
  }
}

// Run demo
if (require.main === module) {
  main();
}

module.exports = {
  getDAGTips,
  getBlock,
  submitTransaction,
  subscribeUTXO,
  healthCheck
};
