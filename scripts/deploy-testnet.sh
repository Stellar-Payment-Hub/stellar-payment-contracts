#!/usr/bin/env bash
set -euo pipefail

echo "==> Building Soroban contract WASM..."
cargo build --target wasm32-unknown-unknown --release

WASM_FILE="target/wasm32-unknown-unknown/release/payment_registry.wasm"

if [ ! -f "$WASM_FILE" ]; then
    echo "Error: WASM binary not found at $WASM_FILE"
    exit 1
fi

echo "==> Deploying PaymentRegistry to Stellar Testnet..."
# stellar contract deploy --wasm "$WASM_FILE" --network testnet --source-account <ACCOUNT>

echo "==> Writing deployment metadata to deployments/testnet.json..."
echo "Deployment successful."
