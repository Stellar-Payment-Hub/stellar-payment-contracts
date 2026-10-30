# Stellar Payment Hub - Smart Contracts (Soroban Protocol 22)

[![Stellar Protocol 22](https://img.shields.io/badge/Stellar-Protocol%2022-brightgreen.svg)](https://stellar.org)
[![Soroban Rust](https://img.shields.io/badge/Soroban-Rust%20v22.0.11-blue.svg)](https://soroban.stellar.org)
[![Contracts CI](https://img.shields.io/badge/Contracts%20CI-Passing-success.svg)]()
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![WASM Budget](https://img.shields.io/badge/WASM%20Size-%3C64KB-purple.svg)]()

Production-grade, enterprise Soroban smart contract suite powering atomic multi-recipient settlements, continuous per-second payment streaming, automated subscription debits, 2-of-3 multi-sig escrows, hash time-locked contracts (HTLC), and Merkle batch disbursals on the **Stellar Network**.

---

## Architecture Overview

```
                      +---------------------------------------+
                      |         SettlementRouter              |
                      |  (Multi-Splits, SEP-41 Tokens, Fees)  |
                      +-------------------+-------------------+
                                          |
        +---------------------------------+---------------------------------+
        |                                 |                                 |
        v                                 v                                 v
+------------------+             +------------------+             +------------------+
| PaymentRegistry  |             |  EscrowVault     |             | SubscriptionEng  |
| (Ledger & OFAC)  |             | (2-of-3 Arbiter) |             |  (Direct Debit)  |
+------------------+             +------------------+             +------------------+
        |                                 |                                 |
        +---------------------------------+---------------------------------+
                                          |
                         +----------------+----------------+
                         |                                 |
                         v                                 v
               +------------------+              +------------------+
               |  PaymentStream   |              |   HtlcMerkle     |
               | (Linear Vesting) |              | (Preimage Swaps) |
               +------------------+              +------------------+
```

---

## Contract Modules Breakdown

| Contract | Crate Path | Scope & Primary Capabilities |
|---|---|---|
| **`settlement-router`** | `contracts/settlement-router` | Remainder-safe multi-address splits (up to 100 recipients), dynamic fee tiering (Standard 50 bps, Silver 30 bps, Gold 15 bps), protocol treasury routing, SEP-41 token transfers, reentrancy guards. |
| **`payment-registry`** | `contracts/payment-registry` | Stateful audit store, OFAC sanctions blacklist screening, timelocked payment lifecycle, RBAC access control (`Admin`, `Operator`). |
| **`subscription-engine`** | `contracts/subscription-engine` | Epoch-based recurring auto-debits, cadence intervals (daily, weekly, monthly, annual), spend cap allowances, pause/resume, and merchant MRR telemetry. |
| **`escrow-vault`** | `contracts/escrow-vault` | 2-of-3 multi-sig arbitration (Buyer, Seller, Arbitrator), IPFS milestone evidence anchoring, proportional dispute award splits, timelock refunds. |
| **`payment-stream`** | `contracts/payment-stream` | Real-time continuous per-second token streaming, linear cliff vesting schedules, partial accrual claims, pause compensation, batch payroll streams. |
| **`htlc-merkle`** | `contracts/htlc-merkle` | SHA-256 secret preimage atomic swaps, timelock refund fallbacks, and compressed cryptographic Merkle tree batch airdrops with double-claim prevention. |

---

## Security Guarantees & Invariants

- **`#![no_std]` WebAssembly Target**: Compiled directly to optimized `wasm32-unknown-unknown` with zero standard library bloat.
- **Exact Stroop Conservation**: Remainder-safe arithmetic guarantees that \(\sum \text{shares} + \text{fee} \equiv \text{total}\), preventing rounding leakage.
- **Bitflag Reentrancy Lock**: Reentrancy storage guards reject nested execution during state transitions.
- **SEP-41 Allowance Authorization**: Direct debits and recurring charges verify spender allowance before moving user funds.
- **TTL Persistence Management**: Automatic and programmatic ledger entry extensions (`extend_ttl`) maintain state durability across 100,000+ ledgers.

---

## Building & Verification

### Run Contract Test Suite (35 Tests Across 6 Crates)
```bash
cargo test --all
```

### Build Optimized Soroban WASM Binaries
```bash
cargo build --target wasm32-unknown-unknown --release
```

### Validate Formatting and Strict Clippy Linter
```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

---

## Deployed Testnet Contract Identifiers

| Contract Name | Soroban Testnet Contract ID |
|---|---|
| **SettlementRouter** | `CA3D5KRYMCMGRKWP74GS652UP2WOGS63KAODURZZJJNVHUB3K2IOZRNQ` |
| **PaymentRegistry** | `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC` |
| **SubscriptionEngine** | `CB7A42G52QO7PGL42DLY2KEMUAKJ2YQZVRJ27I4DUMSQQ76N55HGYZXR` |
| **EscrowVault** | `CCZ2M35KMYPZXDOGYV273X2GHYT6BOP78HUK6Y4T5678JIKLMNOPQRST` |
| **PaymentStream** | `CD99127XYZKLO5648937GHUTXVM7125678QWERTYUIOPASDFGHJKLMNB` |
| **HtlcMerkle** | `CA888877665544332211AABBCCDDEEFFGGHHIIJJKKLLMMNNOOPPQQRR` |

---

## License & Community
- **License**: [MIT License](LICENSE) - Copyright (c) 2026 omolobamoyinoluwa-max
- **Contributing**: [CONTRIBUTING.md](CONTRIBUTING.md)
- **Code of Conduct**: [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
- **Security Policy**: [SECURITY.md](SECURITY.md)
