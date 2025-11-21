#!/bin/bash
set -e
dfx stop

CANISTER="mempool_chess_backend"

echo "🚀 Starting local replica..."
dfx stop --network local >/dev/null 2>&1 || true
dfx start --clean --background
sleep 3

echo "📦 Creating canisters..."
dfx canister create --all

echo "📦 Building canisters..."
dfx build

echo "🚀 Deploying canisters..."
dfx deploy

echo "⏳ Starting round..."
dfx canister call $CANISTER admin_start_round

echo "👤 trader1 placing BUY..."
dfx identity use trader1
dfx canister call $CANISTER submit_order \
  '(variant { Buy }, variant { BTC }, 200000, 10000, vec {}, "")'

echo "👤 trader2 placing SELL..."
dfx identity use trader2
dfx canister call $CANISTER submit_order \
  '(variant { Sell }, variant { BTC }, 200000, 9000, vec {}, "")'

echo "👤 trader3 placing BUY..."
dfx identity use trader3
dfx canister call $CANISTER submit_order \
  '(variant { Buy }, variant { BTC }, 100000, 11000, vec {}, "")'

echo "👤 trader4 placing SELL..."
dfx identity use trader4
dfx canister call $CANISTER submit_order \
  '(variant { Sell }, variant { BTC }, 100000, 8500, vec {}, "")'

echo "🧮 Running clearing..."
dfx canister call $CANISTER admin_run_clearing

echo "✅ DEMO COMPLETE"
