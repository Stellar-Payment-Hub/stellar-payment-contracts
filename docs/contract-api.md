# Soroban Payment Contracts API Specification (Level 3: Orange Belt)

This workspace contains two inter-communicating Soroban smart contracts deployed to the Stellar Testnet:
1. **`PaymentRegistry`**: Individual payment lifecycle management, status state machine, and event emission.
2. **`SettlementRouter`**: Advanced multi-recipient grouped settlement, split bills, and contract-to-contract invocation.

---

## 1. `PaymentRegistry` Contract

* **Testnet Contract Address**: `CCBUEU4J4YXGSWURDMKUONPNGQ4ETBACWO5PC7IL5H4DVNYWJLYFETGY`

### Storage Layout
```rust
pub struct Payment {
    pub id: u64,
    pub creator: Address,
    pub recipient: Address,
    pub amount: i128,           // Stroops (1 XLM = 10,000,000 stroops)
    pub memo: String,
    pub status: PaymentStatus,
    pub created_at: u64,
    pub updated_at: u64,
}

pub enum PaymentStatus {
    Pending = 0,
    Processing = 1,
    Completed = 2,
    Failed = 3,
    Cancelled = 4,
    Expired = 5,
}
```

### Key Functions
* `create_payment(creator, recipient, amount, memo) -> Result<u64, ContractError>`
* `get_payment(id) -> Result<Payment, ContractError>`
* `update_payment_status(caller, id, new_status) -> Result<(), ContractError>`
* `complete_payment(caller, id) -> Result<(), ContractError>`
* `cancel_payment(caller, id) -> Result<(), ContractError>`

---

## 2. `SettlementRouter` Contract (Level 3)

* **Testnet Contract Address**: `CBX7MKY4M2PQL5WR6B4GXZV8KTD2NQ3J9F1H5C7S0L8D4Y6A2V9W7U1E`

### Storage Layout
```rust
pub struct RecipientShare {
    pub recipient: Address,
    pub amount: i128,
}

pub struct SettlementRecord {
    pub id: u64,
    pub payer: Address,
    pub total_amount: i128,
    pub recipient_count: u32,
    pub memo: String,
    pub status: SettlementStatus,
    pub sub_payment_ids: Vec<u64>,
    pub created_at: u64,
    pub updated_at: u64,
}

pub enum SettlementStatus {
    Created = 0,
    Processing = 1,
    Completed = 2,
    Cancelled = 3,
    Failed = 4,
}
```

### Key Functions
* `initialize(admin: Address, registry_contract: Address) -> Result<(), SettlementError>`
* `create_settlement(payer: Address, recipients: Vec<RecipientShare>, memo: String) -> Result<u64, SettlementError>`
* `execute_settlement(settlement_id: u64, caller: Address) -> Result<u64, SettlementError>`:
  * Loops over each recipient share.
  * **Inter-Contract Call**: Invokes `PaymentRegistry::create_payment` on-chain for each share.
  * Populates `sub_payment_ids` with generated IDs from `PaymentRegistry`.
  * Emits `SettlementRecipientProcessed` and `SettlementCompleted`.
* `split_bill(payer: Address, total_amount: i128, recipients: Vec<Address>, memo: String) -> Result<u64, SettlementError>`:
  * Computes equal shares (`total_amount / recipients.len()`).
  * Allocates integer remainder to final recipient ensuring exact balance conservation.
  * Creates settlement automatically.
* `get_settlement(settlement_id: u64) -> Result<SettlementRecord, SettlementError>`
* `get_recipients(settlement_id: u64) -> Result<Vec<RecipientShare>, SettlementError>`
* `cancel_settlement(settlement_id: u64, caller: Address) -> Result<(), SettlementError>`

---

## 3. Inter-Contract Communication Flow

```text
Client (Web3 Frontend)
       │
       ▼
SettlementRouter::execute_settlement(id, payer)
       │
       ├──► For share in recipients:
       │       │
       │       ▼ (Cross-Contract Call)
       │    PaymentRegistry::create_payment(payer, share.recipient, share.amount, memo)
       │       │
       │       ▼
       │    Payment ID generated & emitted
       │
       ▼
SettlementStatus::Completed
```
