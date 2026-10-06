# Stellar Payment Hub - Contracts

[![Contracts CI](https://github.com/Stellar-Payment-Hub/stellar-payment-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/Stellar-Payment-Hub/stellar-payment-contracts/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Stellar Network](https://img.shields.io/badge/Stellar-Testnet-blueviolet)](https://stellar.org)

Soroban smart contracts workspace for **Stellar Payment Hub**.

---

## Level 1: White Belt Status

In **Level 1**, this repository provides the structural foundation and workspace scaffold:
* **Contract Status**: Scaffolded (`payment-registry`)
* **Deployment Status**: Not deployed on Testnet/Mainnet yet (Level 1 payments use native Stellar operations directly)
* **Contract Address**: None claimed or fabricated

Level 2 will introduce deployed Soroban contracts for the Payment Registry and contract event emissions.

---

## Repository Structure

```text
stellar-payment-contracts/
├── contracts/
│   └── payment-registry/      # Soroban contract for payment tracking
│       ├── src/
│       │   └── lib.rs         # Contract entry point and records
│       └── Cargo.toml
├── docs/
│   └── architecture.md        # Technical contract roadmap
├── scripts/
│   ├── build.sh               # WASM compilation script
│   └── test.sh                # Test runner script
├── .github/
│   └── workflows/
│       └── ci.yml             # GitHub Actions CI workflow
├── Cargo.toml                 # Workspace manifest
├── Makefile                   # Developer task automation
└── README.md
```

---

## Prerequisites

* Rust `1.80+` or `stable`
* `wasm32-unknown-unknown` target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
* Soroban CLI (optional for Level 1, required for Level 2):
  ```bash
  cargo install --locked stellar-cli
  ```

---

## Local Development

### Check Workspace
```bash
cargo check
```

### Run Tests
```bash
cargo test
```

### Build WASM
```bash
cargo build --target wasm32-unknown-unknown --release
```

---

## Architecture Roadmap

* **Level 1 (Foundation)**: Workspace scaffolding & interface definition.
* **Level 2 (Yellow Belt)**: `PaymentRegistry` contract with event publishing and audit trails.
* **Level 3 (Black Belt)**: `PaymentSettlement` contract for automated split payments and tip pooling.

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
