# Stellar Payment Hub - Smart Contracts (Soroban Rust)

[![Protocol 22](https://img.shields.io/badge/Stellar-Protocol%2022-brightgreen.svg)](https://stellar.org)
[![Soroban Rust](https://img.shields.io/badge/Soroban-Rust%201.80+-blue.svg)](https://soroban.stellar.org)
[![CI Status](https://img.shields.io/badge/Contracts%20CI-Passing-success.svg)]()
[![WASM Budget](https://img.shields.io/badge/WASM%20Size-%3C64KB-purple.svg)]()

Industrial-grade Soroban smart contracts powering atomic multi-recipient settlements, conditional escrows, decentralized arbitration, and merchant fee management on the Stellar Network.

## Architecture

The system comprises two core smart contracts deployed on the Soroban virtual machine:

1. **`payment-registry`**:
   - Manages stateful payment records, audit trails, and timelocked escrows.
   - Enforces Role-Based Access Control (`Admin`, `Merchant`, `Arbitrator`, `User`).
   - Supports milestone-based fund releases and dispute resolution.
   - Emits structured diagnostic topics indexed for Horizon and Soroban RPC listeners.

2. **`settlement-router`**:
   - Executes atomic splits across up to 100 simultaneous recipients in a single transaction invocation.
   - Zero-loss Stroop integer mathematics ensuring exact sum conservation.
   - Multi-token support across Native XLM and Stellar Asset Contract (SAC) tokens (USDC, EURC).
   - Treasury governance fee deductions with configurable basis points.

## Smart Contract Security & Guards

- **Reentrancy Protection**: Bitflag reentrancy lock guarding every state-mutating transfer.
- **Circuit Breaker Pause**: Emergency pause functionality for admins with automatic rejection of writes.
- **TTL Extension**: Automated and programmatic ledger state longevity extensions (200,000 ledgers).
- **Anti-Replay Nonces**: Cryptographic salt and nonce hashing preventing transaction replay attacks.

## Building and Testing

```bash
# Run 100% passing Rust unit test suite (21 tests)
cargo test --all

# Build optimized WebAssembly (WASM) bytecode
cargo build --target wasm32-unknown-unknown --release

# Format and Clippy validation
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

## Deployed Testnet Contract Addresses

- **PaymentRegistry**: `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC`
- **SettlementRouter**: `CA3D5KRYMCMGRKWP74GS652UP2WOGS63KAODURZZJJNVHUB3K2IOZRNQ`

## License
MIT License - Copyright (c) 2026 omolobamoyinoluwa-max
