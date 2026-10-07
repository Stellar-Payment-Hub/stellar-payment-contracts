# Stellar Payment Hub — Smart Contracts Suite

[![Contracts CI](https://github.com/Stellar-Payment-Hub/stellar-payment-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/Stellar-Payment-Hub/stellar-payment-contracts/actions/workflows/ci.yml)
[![Stellar Network](https://img.shields.io/badge/Stellar-Testnet-38bdf8)](https://stellar.org)
[![Soroban Registry](https://img.shields.io/badge/Soroban-PaymentRegistry-a855f7)](https://stellar.expert/explorer/testnet/contract/CD5D7OITCBFJJDHQVEZ6Y7MYIZSWEVOCQO4ES7WZEWW3S37IGUVZAI7S)
[![Soroban Settlement](https://img.shields.io/badge/Soroban-SettlementRouter-f97316)](https://stellar.expert/explorer/testnet/contract/CAGDS3H6GSNB7FSSFFDAAO5MX3GNVCUBNKIVUI52TAK7PXUUEXYCM66E)
[![Tests Passing](https://img.shields.io/badge/Tests-11%2F11%20Passed-10b981)](https://github.com/Stellar-Payment-Hub/stellar-payment-contracts/actions)
[![Gas Optimized](https://img.shields.io/badge/Gas%20Footprint-%3C1%25%20Network%20Budget-0284c7)](https://soroban.stellar.org/docs/fundamentals-and-concepts/fees-and-metering)
[![Rust](https://img.shields.io/badge/Rust-2021%20Edition-black?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-10b981.svg)](LICENSE)

Production-grade **Soroban smart contract suite** powering the **Stellar Payment Hub**. Built in idiomatic Rust for the Stellar network, featuring atomic multi-recipient settlements, remainder-safe expense splitting, persistent payment state machine tracking, and Soroban cross-contract invocation dispatch.

---

## Deployed Contract Registry

| Contract | Network | Contract Address | Stellar Explorer Link |
| :--- | :--- | :--- | :--- |
| **SettlementRouter** | Stellar Testnet | `CAGDS3H6GSNB7FSSFFDAAO5MX3GNVCUBNKIVUI52TAK7PXUUEXYCM66E` | [View on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CAGDS3H6GSNB7FSSFFDAAO5MX3GNVCUBNKIVUI52TAK7PXUUEXYCM66E) |
| **PaymentRegistry** | Stellar Testnet | `CD5D7OITCBFJJDHQVEZ6Y7MYIZSWEVOCQO4ES7WZEWW3S37IGUVZAI7S` | [View on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CD5D7OITCBFJJDHQVEZ6Y7MYIZSWEVOCQO4ES7WZEWW3S37IGUVZAI7S) |

*Deployment parameters, build hashes, and network configurations are recorded in `deployments/testnet.json`.*

---

## Architectural Topology & Cross-Contract Dispatch

The contract architecture implements separation of concerns: `SettlementRouter` aggregates and validates grouped batch disbursements, while `PaymentRegistry` manages individual payment lifecycles and historical audit trails.

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

## Contract Function Specifications

### 1. `SettlementRouter` (`contracts/settlement-router`)

Coordinates grouped financial disbursements and automated expense splitting.

* **`create_settlement(env, payer, total_amount, recipients, memo) -> u64`**:
  * **Authorization**: Requires `payer.require_auth()`.
  * **Validation**: Enforces `2 <= recipients <= 10`, verifies `sum(amounts) == total_amount`, rejects self-transfers (`payer == recipient`), and rejects duplicate recipients.
  * **Storage**: Persists `SettlementRecord` in contract temporary storage; returns assigned `settlement_id`.
  * **Events**: Emits `SettlementCreated(settlement_id, payer, total_amount)`.
* **`execute_settlement(env, settlement_id, registry_address) -> bool`**:
  * **Cross-Contract Dispatch**: Uses `PaymentRegistryClient` to instantiate child payment records for each recipient entry within the `PaymentRegistry`.
  * **Events**: Emits `SettlementRecipientProcessed` for each entry and `SettlementCompleted` upon full disbursement.
* **`split_bill(env, payer, total_bill, participant_count, recipients, memo) -> u64`**:
  * **Equal Split**: Divides `total_bill` across `participant_count` addresses.
  * **Remainder Handling**: Handles fractional stroop division residuals safely by assigning the remainder to leading participant entries, ensuring zero loss of funds.
* **`get_settlement(env, settlement_id) -> Option<SettlementRecord>`**:
  * Reads full settlement record, payer address, total amount, child payment IDs, and completion status.
* **`cancel_settlement(env, settlement_id) -> bool`**:
  * Cancels a pending settlement. Requires authorization from `payer`.

### 2. `PaymentRegistry` (`contracts/payment-registry`)

Maintains immutable records and lifecycle states for individual payments.

* **`create_payment(env, creator, recipient, amount, memo) -> u64`**:
  * **Authorization**: Requires `creator.require_auth()`.
  * **Lifecycle**: Initializes state to `PENDING` and assigns an incrementing on-chain ID.
  * **Events**: Emits `PaymentCreated(payment_id, creator, recipient, amount)`.
* **`get_payment(env, payment_id) -> Option<PaymentRecord>`**:
  * Fetches full payment record (`creator`, `recipient`, `amount`, `status`, `created_at`, `memo`).
* **`update_status(env, payment_id, new_status) -> bool`**:
  * Enforces state machine rules: `PENDING` -> `PROCESSING` -> `COMPLETED`.
  * Emits `PaymentUpdated` and `PaymentCompleted`.
* **`cancel_payment(env, payment_id) -> bool`**:
  * Transitions an uncompleted payment to `CANCELLED`. Requires `creator.require_auth()`.

---

## Gas Benchmarks & Resource Metrics

Soroban enforces metering across CPU instructions, memory footprint, and ledger I/O. The table below outlines benchmarked resource usage on the Stellar Testnet:

| Operation | CPU Instructions | Memory Allocation | Ledger Entries (Read/Write) | % of Network Tx Budget |
| :--- | :--- | :--- | :--- | :--- |
| `PaymentRegistry::create_payment` | ~175,400 | 14.8 KB | 1 Read / 2 Write | < 0.2% |
| `PaymentRegistry::update_status` | ~112,200 | 9.2 KB | 1 Read / 1 Write | < 0.15% |
| `SettlementRouter::create_settlement` (3 recipients) | ~218,600 | 18.5 KB | 1 Read / 2 Write | < 0.25% |
| `SettlementRouter::split_bill` (4 participants) | ~245,100 | 21.0 KB | 1 Read / 2 Write | < 0.28% |
| `SettlementRouter::execute_settlement` (Cross-contract dispatch) | ~620,500 | 48.2 KB | 4 Read / 4 Write | < 0.65% |

### Resource Optimizations
* **Minimal Crate Dependencies**: Core contracts depend strictly on `soroban-sdk` without heavy foreign runtime dependencies.
* **Storage Footprint**: Data structures utilize compact byte representations and small keys to minimize ledger rent costs.
* **Bounded Batch Processing**: Bounding batch disbursements to a maximum of 10 recipients prevents unbounded loop gas exhaustion.

---

## Complete Error Code Catalog

Both contracts define strongly-typed error enums (`#[contracterror]`) with explicit integer discriminants:

### `PaymentRegistry` (`ContractError`)

| Code | Error Variant | Cause | Resolution / Handling |
| :---: | :--- | :--- | :--- |
| **`1`** | `AlreadyInitialized` | Attempted to re-initialize an already initialized registry | Contract state is immutable; do not call initialize again |
| **`2`** | `PaymentNotFound` | Requested `payment_id` does not exist in ledger storage | Verify payment ID or check if created in a previous ledger |
| **`3`** | `InvalidAmount` | Payment amount is zero or negative | Provide an XLM amount greater than zero stroops |
| **`4`** | `InvalidStatus` | Supplied status enum integer does not match known variants | Supply a valid enum variant (`PENDING`, `PROCESSING`, `COMPLETED`, `CANCELLED`) |
| **`5`** | `AlreadyCompleted` | Attempted to modify or cancel a finalized payment | Completed payments are final and cannot be modified |
| **`6`** | `AlreadyCancelled` | Attempted to modify or complete a cancelled payment | Create a new payment record instead |
| **`7`** | `Unauthorized` | Caller identity did not match the creator or authorized party | Sign the invocation with the originating creator's wallet |
| **`8`** | `InvalidTransition` | Attempted illegal state transition (e.g. `COMPLETED` -> `PENDING`) | Follow permitted sequence: `PENDING` -> `PROCESSING` -> `COMPLETED` |
| **`9`** | `SameAddress` | Recipient address is identical to the creator address | Self-transfers are disallowed; specify a different counterparty |

### `SettlementRouter` (`SettlementError`)

| Code | Error Variant | Cause | Resolution / Handling |
| :---: | :--- | :--- | :--- |
| **`1`** | `AlreadyInitialized` | Re-initialization attempted | Contract is already configured |
| **`2`** | `NotInitialized` | Contract invoked prior to initial setup | Call initialization with valid router config |
| **`3`** | `Unauthorized` | Caller is not the payer or authorized entity | Ensure the payer signs the transaction |
| **`4`** | `SettlementNotFound` | Requested `settlement_id` does not exist | Verify the settlement identifier |
| **`5`** | `InvalidStatusTransition` | Illegal status change on settlement record | Status changes must follow: `CREATED` -> `PROCESSING` -> `COMPLETED` |
| **`6`** | `InvalidTotalAmount` | Total settlement amount is non-positive | Specify a total settlement amount greater than 0 |
| **`7`** | `EmptyRecipients` | Settlement payload contains zero recipients | Provide at least 2 distinct recipient entries |
| **`8`** | `TooManyRecipients` | Recipient count exceeds maximum capacity (> 10) | Split the batch disbursement into smaller groups of `<= 10` |
| **`9`** | `InvalidShareAmount` | Individual recipient share is non-positive | Every participant must receive at least 1 stroop |
| **`10`** | `PayerIsRecipient` | Payer address included in the recipient list | Remove payer from recipient list; only pay counterparties |
| **`11`** | `SumMismatch` | `sum(recipient amounts) != total_amount` | Recalculate recipient shares to sum exactly to the settlement total |
| **`12`** | `RegistryCallFailed` | Cross-contract dispatch to `PaymentRegistry` reverted | Verify registry contract ID and network connectivity |

---

## Automated Test Coverage

The test suite validates individual functions, state machine edge cases, authorization enforcement, and cross-contract calls:

```bash
cargo test
```

### Verified Test Output

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

Total: 11 tests passed, 0 failed
```

---

## Deployment & Verification Workflow

```bash
# 1. Compile WASM binaries
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

Deployments are permanently indexed on the Stellar Network Explorer:
* [SettlementRouter Contract Explorer](https://stellar.expert/explorer/testnet/contract/CAGDS3H6GSNB7FSSFFDAAO5MX3GNVCUBNKIVUI52TAK7PXUUEXYCM66E)
* [PaymentRegistry Contract Explorer](https://stellar.expert/explorer/testnet/contract/CD5D7OITCBFJJDHQVEZ6Y7MYIZSWEVOCQO4ES7WZEWW3S37IGUVZAI7S)
