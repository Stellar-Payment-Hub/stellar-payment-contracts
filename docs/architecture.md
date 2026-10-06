# Architecture: Stellar Payment Contracts

## Overview
This repository hosts the smart contract foundations for the **Stellar Payment Hub**.

### Progression Across Levels

* **Level 1 (White Belt - Current)**:
  - Repository structure and workspace scaffolding.
  - Contract workspace setup for `payment-registry`.
  - Level 1 does **not** deploy contracts to Testnet or Mainnet; payments in Level 1 use native Stellar Testnet payment operations directly.
  - No dummy contract addresses are claimed or fabricated.

* **Level 2 (Yellow Belt - Planned)**:
  - Implementation of the `PaymentRegistry` contract on Soroban.
  - Event emission (`payment_registered`) for payment indexing.
  - Contract calls integrated into the payment flow alongside direct payments.

* **Level 3 (Black Belt - Planned)**:
  - `PaymentSettlement` contract for automated split bills and escrow-less multi-address distributions.
  - Inter-contract calls and role-based permissions.

## Contract Workspace Structure
```text
stellar-payment-contracts/
├── contracts/
│   └── payment-registry/
│       ├── src/
│       │   └── lib.rs
│       └── Cargo.toml
├── docs/
│   └── architecture.md
├── scripts/
│   ├── build.sh
│   └── test.sh
├── .github/
│   └── workflows/
│       └── ci.yml
├── Cargo.toml
├── Makefile
└── README.md
```
