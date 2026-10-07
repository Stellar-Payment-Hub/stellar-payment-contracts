# Architecture: Stellar Payment Contracts

## Overview
This repository hosts the production Soroban smart contract suite for the **Stellar Payment Hub**.

The suite provides decentralized financial settlement primitives directly on the Stellar blockchain:
1. **`PaymentRegistry`**: Persistent state machine for individual payment lifecycle tracking (`PENDING` -> `PROCESSING` -> `COMPLETED` / `CANCELLED`), non-repudiation, and topic-based ledger event indexing.
2. **`SettlementRouter`**: Programmable multi-recipient settlement engine supporting batch payouts (2 to 10 recipients), remainder-safe bill splitting with integer stroop preservation, and cross-contract dispatch to `PaymentRegistry`.

---

## Architectural Topology & Inter-Contract Dispatch

```text
┌────────────────────────────────────────────────────────────────────────┐
│                        SettlementRouter Contract                       │
│              CAGDS3H6GSNB7FSSFFDAAO5MX3GNVCUBNKIVUI52TAK7PXUUEXYCM66E  │
│                                                                        │
│  • Batch recipient validation     • Mathematical invariant check       │
│  • Remainder-safe bill splitting  • Atomic multi-recipient execution   │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                     Cross-Contract Invocations
                 PaymentRegistryClient::create_payment
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                        PaymentRegistry Contract                        │
│              CD5D7OITCBFJJDHQVEZ6Y7MYIZSWEVOCQO4ES7WZEWW3S37IGUVZAI7S  │
│                                                                        │
│  • Granular payment lifecycle     • Replay-protected counter storage   │
│  • State transitions & audit      • Persistent topic event emissions   │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
                     Stellar Ledger Topic Events
           (SettlementCreated, PaymentCreated, StatusUpdated)
```

---

## Contract Workspace Structure
```text
stellar-payment-contracts/
├── contracts/
│   ├── payment-registry/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── storage.rs
│   │   │   ├── types.rs
│   │   │   └── test.rs
│   │   └── Cargo.toml
│   └── settlement-router/
│       ├── src/
│       │   ├── lib.rs
│       │   ├── storage.rs
│       │   ├── types.rs
│       │   └── test.rs
│       └── Cargo.toml
├── deployments/
│   └── testnet.json
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

---

## Key Guarantees
* **Deterministic Integer Math**: Indivisible stroop remainders (`1 XLM = 10,000,000 stroops`) from bill division are assigned to leading recipients, ensuring zero rounding leakage.
* **Cryptographic Replay Protection**: Incrementing on-chain sequence IDs and authorization verification prevent duplicate disbursements.
* **Storage Optimization**: Minimal foot-print TTL state storage adheres to strict Soroban rent models.
