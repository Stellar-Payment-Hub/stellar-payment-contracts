# Soroban Payment Registry Contract Events

## Event Streams

The `PaymentRegistry` contract emits structured Soroban events whenever payment lifecycles change. These events are consumed by the **`stellar-payment-backend`** event listener service to provide real-time updates to frontend clients.

---

## 1. `PaymentCreated`
Emitted immediately when a new payment request is registered on-chain.

* **Topics**:
  1. `Symbol("payment")`
  2. `Symbol("created")`
* **Data Payload**:
  * `id: u64` (Payment identifier)
  * `creator: Address` (Sender/Payer)
  * `recipient: Address` (Beneficiary)
  * `amount: i128` (Stroops)

---

## 2. `PaymentUpdated`
Emitted when a payment status transitions.

* **Topics**:
  1. `Symbol("payment")`
  2. `Symbol("updated")`
* **Data Payload**:
  * `id: u64`
  * `status: PaymentStatus`
  * `updated_at: u64` (Ledger timestamp)

---

## 3. `PaymentCompleted`
Emitted when a payment settles successfully.

* **Topics**:
  1. `Symbol("payment")`
  2. `Symbol("completed")`
* **Data Payload**:
  * `id: u64`
  * `updated_at: u64`

---

## 4. `PaymentCancelled`
Emitted when a payment is cancelled by creator or recipient prior to completion.

* **Topics**:
  1. `Symbol("payment")`
  2. `Symbol("cancelled")`
* **Data Payload**:
  * `id: u64`
  * `updated_at: u64`
