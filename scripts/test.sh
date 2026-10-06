#!/usr/bin/env bash
set -euo pipefail

echo "Running Soroban contract unit tests..."
cargo test
echo "Tests completed successfully."
