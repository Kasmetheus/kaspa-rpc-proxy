#!/usr/bin/env python3
"""
Kaspa RPC Service - Example Client (Python)

Demonstrates how to interact with all 4 core endpoints

Requirements: pip install requests websocket-client
"""

import json
import time
import requests
from websocket import create_connection

BASE_URL = "http://localhost:8080"
WS_URL = "ws://localhost:8080"


def health_check():
    """Check service health"""
    print("🏥 Checking service health...")
    
    response = requests.get(f"{BASE_URL}/health")
    
    if response.status_code == 200:
        print("✅ Service is healthy")
        return True
    else:
        print("❌ Service unhealthy")
        return False


def get_dag_tips():
    """Get DAG tips (virtual selected parent chain)"""
    print("\n📊 Getting DAG tips...")
    
    response = requests.post(
        f"{BASE_URL}/rpc/getDAGTips",
        headers={"Content-Type": "application/json"},
        json={}
    )
    
    data = response.json()
    
    if data.get("success"):
        dag_data = data["data"]
        print(f"✅ Block count: {dag_data['blockCount']}")
        print(f"✅ Virtual DAA score: {dag_data['virtualDaaScore']}")
        print(f"⏱️  Latency: {data['latency_ms']}ms")
        print(f"📍 Tips: {', '.join(dag_data['tipHashes'][:2])}...")
        return dag_data
    else:
        print(f"❌ Error: {data.get('error')}")
        return None


def get_block(block_hash):
    """Get block by hash"""
    print(f"\n🧱 Getting block {block_hash[:16]}...")
    
    response = requests.post(
        f"{BASE_URL}/rpc/getBlock",
        headers={"Content-Type": "application/json"},
        json={
            "hash": block_hash,
            "includeTransactions": True
        }
    )
    
    data = response.json()
    
    if data.get("success"):
        block = data["data"]
        print(f"✅ Block hash: {block['hash'][:16]}...")
        print(f"✅ DAA score: {block['header']['daaScore']}")
        print(f"✅ Transactions: {len(block['transactions'])}")
        print(f"⏱️  Latency: {data['latency_ms']}ms")
        return block
    else:
        print(f"❌ Error: {data.get('error')}")
        return None


def submit_transaction(transaction):
    """Submit transaction to network"""
    print("\n📤 Submitting transaction...")
    
    response = requests.post(
        f"{BASE_URL}/rpc/submitTransaction",
        headers={"Content-Type": "application/json"},
        json={
            "transaction": transaction,
            "allowOrphan": False
        }
    )
    
    data = response.json()
    
    if data.get("success"):
        print(f"✅ Transaction ID: {data['data']['transactionId']}")
        print(f"⏱️  Latency: {data['latency_ms']}ms")
        return data["data"]["transactionId"]
    else:
        print(f"❌ Error: {data.get('error')}")
        return None


def subscribe_utxo(addresses, duration=60):
    """Subscribe to UTXO changes via WebSocket"""
    print("\n🔔 Subscribing to UTXO changes...")
    print(f"📍 Addresses: {', '.join(addresses)}")
    
    address_params = ",".join(addresses)
    ws_url = f"{WS_URL}/ws/subscribeUTXO?addresses={address_params}"
    
    try:
        ws = create_connection(ws_url)
        print("✅ WebSocket connected")
        
        # Set timeout for receiving messages
        ws.settimeout(5)
        
        start_time = time.time()
        
        while time.time() - start_time < duration:
            try:
                message = ws.recv()
                data = json.loads(message)
                
                if data.get("status") == "subscribed":
                    print("✅ Subscription confirmed")
                elif data.get("type") == "utxo_changed":
                    print("\n🔔 UTXO Change Notification:")
                    print(f"  Added: {len(data['added'])} UTXOs")
                    print(f"  Removed: {len(data['removed'])} UTXOs")
                    
                    # Show details of first added UTXO
                    if data["added"]:
                        utxo = data["added"][0]
                        print(f"  📍 Address: {utxo['address']}")
                        if utxo.get("utxo_entry"):
                            print(f"  💰 Amount: {utxo['utxo_entry']['amount']}")
                
            except Exception as e:
                if "timed out" not in str(e):
                    print(f"⚠️  Receive error: {e}")
                continue
        
        print("\n⏰ Closing subscription...")
        ws.close()
        print("🔌 WebSocket closed")
        
    except Exception as e:
        print(f"❌ WebSocket error: {e}")


def main():
    """Run demo"""
    print("🚀 Kaspa RPC Service - Python Client Demo\n")
    print("=" * 50)
    
    try:
        # 1. Health check
        if not health_check():
            raise Exception("Service not healthy")
        
        # 2. Get DAG tips
        dag_info = get_dag_tips()
        
        # 3. Get a block (if tip hashes available)
        if dag_info and dag_info.get("tipHashes"):
            tip_hash = dag_info["tipHashes"][0]
            get_block(tip_hash)
        
        # 4. Subscribe to UTXO changes
        # Replace with real testnet addresses for live updates
        example_addresses = [
            "kaspa:qztest1234567890abcdef",
            "kaspa:qztest0987654321fedcba"
        ]
        
        print("\n💡 Starting UTXO subscription (will run for 60 seconds)")
        print("   Press Ctrl+C to exit early")
        
        subscribe_utxo(example_addresses, duration=60)
        
        print("\n" + "=" * 50)
        print("✅ Demo complete!\n")
        
    except KeyboardInterrupt:
        print("\n\n⚠️  Interrupted by user")
    except Exception as e:
        print(f"\n❌ Error: {e}")
        print("\n💡 Make sure the service is running:")
        print("   docker-compose up -d\n")
        return 1
    
    return 0


if __name__ == "__main__":
    exit(main())
