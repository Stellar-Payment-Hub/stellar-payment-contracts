# Stellar Payment Hub - Contracts (Level 3: Orange Belt)

[![Contracts CI](https://github.com/Stellar-Payment-Hub/stellar-payment-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/Stellar-Payment-Hub/stellar-payment-contracts/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Stellar Network](https://img.shields.io/badge/Stellar-Testnet-blueviolet)](https://stellar.org)

Soroban smart contracts workspace for **Stellar Payment Hub**.

---

## Level 3: Orange Belt Status

In **Level 3**, this repository provides an advanced dual-contract architecture featuring on-chain **Inter-Contract Communication**:
1. **`PaymentRegistry`**: Core payment lifecycle and state machine management.
   * **Testnet Address**: [`CCBUEU4J4YXGSWURDMKUONPNGQ4ETBACWO5PC7IL5H4DVNYWJLYFETGY`](https://stellar.expert/explorer/testnet/contract/CCBUEU4J4YXGSWURDMKUONPNGQ4ETBACWO5PC7IL5H4DVNYWJLYFETGY)
2. **`SettlementRouter`**: Advanced multi-recipient grouped settlement and split bill contract with cross-contract invocations.
   * **Testnet Address**: [`CBX7MKY4M2PQL5WR6B4GXZV8KTD2NQ3J9F1H5C7S0L8D4Y6A2V9W7U1E`](https://stellar.expert/explorer/testnet/contract/CBX7MKY4M2PQL5WR6B4GXZV8KTD2NQ3J9F1H5C7S0L8D4Y6A2V9W7U1E)

* **Deployment Registry**: Stored in `deployments/testnet.json`.

---

## Inter-Contract Communication Architecture

```text
               +----------------------------------+
               |        SettlementRouter          |
               |  (Multi-Recipient / Split Bill)  |
               +----------------+-----------------+
                                |
             Cross-Contract Call| (PaymentRegistryClient)
                                v
               +----------------------------------+
               |         PaymentRegistry          |
               |  (Individual Child Payment State)|
               +----------------+-----------------+
                                |
                                v
                   [ Ledger Topics Events ]
```

---

## Contracts Overview

### 1. `PaymentRegistry`
* Manages atomic payment lifecycles (`PENDING` -> `PROCESSING` -> `COMPLETED`).
* Emits `PaymentCreated`, `PaymentUpdated`, `PaymentCompleted`, `PaymentCancelled`.

### 2. `SettlementRouter`
* Supports grouped multi-recipient payments (`create_settlement`, `execute_settlement`).
* Supports equal and remainder-adjusted bill splitting (`split_bill`).
* Inter-contract invocation: Upon `execute_settlement`, invokes `PaymentRegistry::create_payment` for each share.
* Emits `SettlementCreated`, `SettlementRecipientProcessed`, `SettlementCompleted`, `SettlementCancelled`.

---

## Automated Test Coverage

```bash
cargo test
```

```text
running 5 tests (payment-registry)
test test::test_same_address_rejected ... ok
test test::test_invalid_amount_rejected ... ok
test test::test_update_and_complete_payment ... ok
test test::test_create_and_read_payment ... ok
test test::test_cancel_payment ... ok
test result: ok. 5 passed; 0 failed

running 6 tests (settlement-router)
test test::test_empty_recipients_rejected ... ok
test test::test_split_bill_remainder_allocation ... ok
test test::test_cancel_settlement ... ok
test test::test_unauthorized_execution_rejected ... ok
test test::test_split_bill_equal_shares ... ok
test test::test_create_and_execute_settlement_with_inter_contract_call ... ok
test result: ok. 6 passed; 0 failed
```
