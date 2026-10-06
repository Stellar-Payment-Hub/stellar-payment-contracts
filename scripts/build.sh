#!/usr/bin/env bash
set -euo pipefail

echo "Building Soroban contracts..."
cargo build --target wasm32-unknown-unknown --release
echo "Contract build completed successfully."
