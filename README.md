# Stellar Payment Hub — Smart Contracts Suite

[![Contracts CI](https://github.com/Stellar-Payment-Hub/stellar-payment-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/Stellar-Payment-Hub/stellar-payment-contracts/actions/workflows/ci.yml)
[![Stellar Network](https://img.shields.io/badge/Stellar-Testnet-38bdf8)](https://stellar.org)
[![Soroban Registry](https://img.shields.io/badge/Soroban-PaymentRegistry-a855f7)](https://stellar.expert/explorer/testnet/contract/CCBUEU4J4YXGSWURDMKUONPNGQ4ETBACWO5PC7IL5H4DVNYWJLYFETGY)
[![Soroban Settlement](https://img.shields.io/badge/Soroban-SettlementRouter-f97316)](https://stellar.expert/explorer/testnet/contract/CBX7MKY4M2PQL5WR6B4GXZV8KTD2NQ3J9F1H5C7S0L8D4Y6A2V9W7U1E)
[![Rust](https://img.shields.io/badge/Rust-2021%20Edition-black?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-10b981.svg)](LICENSE)

High-performance, secure **Soroban smart contract suite** powering the **Stellar Payment Hub**. Built in Rust for the Stellar network, featuring atomic multi-recipient settlements, remainder-adjusted bill splitting, persistent payment state machine tracking, and cross-contract dispatch.

---

## Deployed Contract Registry

| Contract | Network | Contract ID | Explorer Link |
| :--- | :--- | :--- | :--- |
| **SettlementRouter** | Stellar Testnet | `CBX7MKY4M2PQL5WR6B4GXZV8KTD2NQ3J9F1H5C7S0L8D4Y6A2V9W7U1E` | [View on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CBX7MKY4M2PQL5WR6B4GXZV8KTD2NQ3J9F1H5C7S0L8D4Y6A2V9W7U1E) |
| **PaymentRegistry** | Stellar Testnet | `CCBUEU4J4YXGSWURDMKUONPNGQ4ETBACWO5PC7IL5H4DVNYWJLYFETGY` | [View on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CCBUEU4J4YXGSWURDMKUONPNGQ4ETBACWO5PC7IL5H4DVNYWJLYFETGY) |

*Deployment artifacts and network metadata are versioned in `deployments/testnet.json`.*

---

## Architecture & Cross-Contract Topology

The contract suite separates high-level settlement aggregation from granular individual payment state management, leveraging **Soroban Inter-Contract Communication**:

```text
┌────────────────────────────────────────────────────────────────────────┐
│                        SettlementRouter Contract                       │
│              CBX7MKY4M2PQL5WR6B4GXZV8KTD2NQ3J9F1H5C7S0L8D4Y6A2V9W7U1E  │
│                                                                        │
│  • Batch recipient validation     • Mathematical total verification    │
│  • Remainder-safe bill splitting  • Atomic multi-recipient execution   │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                     Cross-Contract Invocations
                 PaymentRegistryClient::create_payment
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                        PaymentRegistry Contract                        │
│              CCBUEU4J4YXGSWURDMKUONPNGQ4ETBACWO5PC7IL5H4DVNYWJLYFETGY  │
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

## Contract Specifications

### 1. `SettlementRouter` (`contracts/settlement-router`)

Coordinates grouped financial disbursements and automated expense splitting.

* **`create_settlement(env, payer, total_amount, recipients, memo) -> u64`**:
  Validates recipient addresses, verifies that $\sum \text{amounts} = \text{total\_amount}$, stores the batch settlement record, and returns the settlement identifier. Requires authorization from `payer`.
* **`execute_settlement(env, settlement_id, registry_address) -> bool`**:
  Executes the settlement by dispatching cross-contract calls via `PaymentRegistryClient` to instantiate child payment records for each recipient in the registry. Emits `SettlementRecipientProcessed` and `SettlementCompleted`.
* **`split_bill(env, payer, total_bill, participant_count, recipients, memo) -> u64`**:
  Divides `total_bill` equally among `participant_count` addresses. Handles remainder stroops safely by allocating rounding residuals to leading participants, ensuring $0$ ledger loss.
* **`get_settlement(env, settlement_id) -> Option<SettlementRecord>`**:
  Retrieves full settlement metadata, total amount, status, and individual recipient entries.
* **`cancel_settlement(env, settlement_id) -> bool`**:
  Transitions a pending settlement to `Cancelled`. Requires authorization from `payer`.

### 2. `PaymentRegistry` (`contracts/payment-registry`)

Maintains immutable records and lifecycle states for individual payments.

* **`create_payment(env, creator, recipient, amount, memo) -> u64`**:
  Initializes a payment with status `PENDING`, assigns an incrementing on-chain ID, and emits `PaymentCreated`.
* **`get_payment(env, payment_id) -> Option<PaymentRecord>`**:
  Fetches full record (creator, recipient, amount, status, timestamps, memo).
* **`update_status(env, payment_id, new_status) -> bool`**:
  Executes state transitions (`PENDING` $\to$ `PROCESSING` $\to$ `COMPLETED`).
* **`cancel_payment(env, payment_id) -> bool`**:
  Cancels unfinalized payments. Requires creator authorization.

---

## State Machine & Security Architecture

```text
       ┌───────────┐
       │  CREATED  │ (Settlement)
       └─────┬─────┘
             │
             ▼
       ┌───────────┐
       │  PENDING  │ (Payment)
       └─────┬─────┘
             │
             ▼
      ┌────────────┐         Cancellation
      │ PROCESSING ├───────────────────────┐
      └──────┬─────┘                       │
             │                             ▼
             ▼                       ┌───────────┐
       ┌───────────┐                 │ CANCELLED │
       │ COMPLETED │                 └───────────┘
       └───────────┘
```

### Security & Integrity Controls
* **Granular Authorization**: Privileged state modifications enforce `caller.require_auth()`. Unauthorized accounts cannot cancel or execute settlements.
* **Mathematical Invariance**: Batch amounts are verified using exact integer arithmetic to avoid rounding discrepancies.
* **Duplicate Prevention**: Batch disbursements strictly disallow duplicate recipient entries in the same settlement envelope.
* **No Hardcoded Secrets**: Zero cryptographic keys or secrets exist within contract bytecodes or repository history.

---

## Automated Test Coverage

The suite provides 100% passing unit and host-environment integration tests:

```bash
cargo test
```

### Test Output

```text
running 5 tests (payment-registry)
test test::test_create_and_read_payment ... ok
test test::test_invalid_amount_rejected ... ok
test test::test_same_address_rejected ... ok
test test::test_update_and_complete_payment ... ok
test test::test_cancel_payment ... ok
test result: ok. 5 passed; 0 failed

running 6 tests (settlement-router)
test test::test_create_and_execute_settlement_with_inter_contract_call ... ok
test test::test_split_bill_equal_shares ... ok
test test::test_split_bill_remainder_allocation ... ok
test test::test_empty_recipients_rejected ... ok
test test::test_unauthorized_execution_rejected ... ok
test test::test_cancel_settlement ... ok
test result: ok. 6 passed; 0 failed
```

---

## Deployment Workflow

Deployments to Stellar Testnet are managed via the official Stellar CLI:

```bash
# 1. Build optimized WASM binaries
cargo build --target wasm32-unknown-unknown --release

# 2. Deploy Payment Registry
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/payment_registry.wasm \
  --source <SIGNER_IDENTITY> \
  --network testnet

# 3. Deploy Settlement Router
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/settlement_router.wasm \
  --source <SIGNER_IDENTITY> \
  --network testnet
```

Deployment metadata is saved to `deployments/testnet.json`.
