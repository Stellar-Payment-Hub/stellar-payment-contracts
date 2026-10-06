# Stellar Payment Hub - Contracts (Level 2: Yellow Belt)

[![Contracts CI](https://github.com/Stellar-Payment-Hub/stellar-payment-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/Stellar-Payment-Hub/stellar-payment-contracts/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Stellar Network](https://img.shields.io/badge/Stellar-Testnet-blueviolet)](https://stellar.org)

Soroban smart contracts workspace for **Stellar Payment Hub**.

---

## Level 2: Yellow Belt Status

In **Level 2**, this repository provides the programmable payment coordination contract:
* **Contract Name**: `PaymentRegistry`
* **Network**: **Stellar Testnet**
* **Deployed Contract Address**: [`CCBUEU4J4YXGSWURDMKUONPNGQ4ETBACWO5PC7IL5H4DVNYWJLYFETGY`](https://stellar.expert/explorer/testnet/contract/CCBUEU4J4YXGSWURDMKUONPNGQ4ETBACWO5PC7IL5H4DVNYWJLYFETGY)
* **Metadata**: Recorded in `deployments/testnet.json`

---

## Contract Overview & Architecture

The `PaymentRegistry` contract manages programmable payment requests and enforces non-custodial lifecycle state transitions on-chain:

```text
               +---------------------------+
               |      create_payment()     |
               +-------------+-------------+
                             |
                             v
                    [ PENDING (0) ]
                      /    |    \
                     /     |     \
                    v      v      v
        [ PROCESSING (1) ] |   [ CANCELLED (4) ]
             |     |       |
             |     \       /
             |      v     v
             |   [ FAILED (3) ]
             v
      [ COMPLETED (2) ]
```

---

## Public Functions

| Function | Parameters | Description |
| :--- | :--- | :--- |
| `initialize` | `admin: Address` | Sets the contract administrator |
| `create_payment` | `creator: Address, recipient: Address, amount: i128, memo: String` | Generates a new payment record (requires `creator` auth) |
| `get_payment` | `id: u64` | Retrieves payment record by ID |
| `update_payment_status` | `caller: Address, id: u64, new_status: PaymentStatus` | Updates payment status with state machine invariant checks |
| `complete_payment` | `caller: Address, id: u64` | Marks payment as completed and immutable |
| `cancel_payment` | `caller: Address, id: u64` | Cancels payment prior to completion |
| `get_payment_count` | *none* | Returns total number of registered payments |

---

## Contract Events

The contract emits structured Soroban events for real-time backend synchronization:
* **`(payment, created)`**: `(id, creator, recipient, amount)`
* **`(payment, updated)`**: `(id, new_status, timestamp)`
* **`(payment, completed)`**: `(id, timestamp)`
* **`(payment, cancelled)`**: `(id, timestamp)`

See [docs/events.md](docs/events.md) for full payload schemas.

---

## Local Development & Testing

### Run Contract Unit Tests
```bash
cargo test
```

### Build WASM Target
```bash
cargo build --target wasm32-unknown-unknown --release
```

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
