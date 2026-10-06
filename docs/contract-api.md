# Soroban Payment Registry Contract API Specification

## Contract Name: `PaymentRegistry`
* **Target Network**: Stellar Testnet
* **Standard**: Soroban Smart Contract Specification

---

## 1. Storage Layout & Data Model

### Payment Data Structure
```rust
pub struct Payment {
    pub id: u64,
    pub creator: Address,
    pub recipient: Address,
    pub amount: i128,           // Amount in stroops (1 XLM = 10,000,000 stroops)
    pub memo: String,           // Optional identifier / invoice note
    pub status: PaymentStatus,  // State machine enum
    pub created_at: u64,        // Ledger timestamp
    pub updated_at: u64,        // Ledger timestamp
}
```

### Status Lifecycle Enum
```rust
pub enum PaymentStatus {
    Pending = 0,     // Initial state awaiting settlement
    Processing = 1,  // Transaction broadcast / in-flight
    Completed = 2,   // Settled on ledger
    Failed = 3,      // Terminal failure
    Cancelled = 4,   // Voided by creator or recipient
    Expired = 5,     // Timed out
}
```

---

## 2. Public Functions

### `initialize(admin: Address) -> Result<(), ContractError>`
Initializes the contract administrator. Can only be invoked once.

### `create_payment(creator: Address, recipient: Address, amount: i128, memo: String) -> Result<u64, ContractError>`
* **Authorization**: Requires `creator.require_auth()`.
* **Validation**:
  * `amount > 0` (or `ContractError::InvalidAmount`).
  * `creator != recipient` (or `ContractError::SameAddress`).
* **Returns**: New incremented `u64` Payment ID.
* **Emits**: `(payment, created)` event.

### `get_payment(id: u64) -> Result<Payment, ContractError>`
Reads a payment record from persistent storage.
* **Returns**: `Ok(Payment)` or `Err(ContractError::PaymentNotFound)`.

### `update_payment_status(caller: Address, id: u64, new_status: PaymentStatus) -> Result<(), ContractError>`
* **Authorization**: Requires `caller.require_auth()`. Caller must be creator, recipient, or admin.
* **State Machine Invariants**:
  * Completed payments cannot transition.
  * Cancelled payments cannot transition.
* **Emits**: `(payment, updated)` event.

### `complete_payment(caller: Address, id: u64) -> Result<(), ContractError>`
Convenience helper to set status to `PaymentStatus::Completed`.

### `cancel_payment(caller: Address, id: u64) -> Result<(), ContractError>`
Convenience helper to set status to `PaymentStatus::Cancelled`.

### `get_payment_count() -> u64`
Returns total number of created payments in this registry.

---

## 3. Error Codes

| Code | Variant | Description |
| :---: | :--- | :--- |
| `1` | `AlreadyInitialized` | Admin has already been established |
| `2` | `PaymentNotFound` | Requested payment ID does not exist |
| `3` | `InvalidAmount` | Amount must be strictly greater than 0 |
| `4` | `InvalidStatus` | Unrecognized status code |
| `5` | `AlreadyCompleted` | Payment is already completed and immutable |
| `6` | `AlreadyCancelled` | Payment is already cancelled and immutable |
| `7` | `Unauthorized` | Caller is not authorized for this record |
| `8` | `InvalidTransition` | Disallowed state transition |
| `9` | `SameAddress` | Recipient cannot be the same as creator |
