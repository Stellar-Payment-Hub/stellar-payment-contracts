# Changelog - Smart Contracts

All notable changes to the `stellar-payment-contracts` repository are documented in this file.

## [1.2.0] - 2026-10-30

### Added
- **Dynamic Fee Tier Engine (Batch 6, Items 101–120)**:
  - Automated volume-based fee tiers: Standard (50 bps), Silver (30 bps), Gold (15 bps).
  - VIP partner discount rates with configurable administrative whitelisting.
  - OFAC sanctions blacklist registry with instant transaction screening.
  - Multi-contract integration test suite certifying interoperability between Registry, Router, and Vaults.

- **Hash Time-Locked Contracts & Merkle Disbursals (Batch 5, Items 81–100)**:
  - Standalone `contracts/htlc-merkle` contract crate.
  - SHA-256 secret preimage atomic swaps with recipient claims and sender timelock refunds.
  - Compressed cryptographic Merkle tree batch airdrops with double-claim bitmap prevention.

- **Real-Time Payment Streaming & Linear Vesting (Batch 4, Items 61–80)**:
  - Standalone `contracts/payment-stream` contract crate.
  - Continuous per-second payment flow with linear vesting cliffs.
  - Partial unaccrued withdrawals, pause compensation calculations, and batch payroll streams.

- **Time-Locked Escrow & 2-of-3 Multi-Sig Arbitration (Batch 3, Items 41–60)**:
  - Standalone `contracts/escrow-vault` contract crate.
  - 2-of-3 multi-sig voting resolution with buyer, seller, and arbitrator quorums.
  - Milestone deliverable submissions anchored to IPFS content CIDs.
  - Arbitrator award split allocations (0%–100%) and automatic statutory commission deductions.

- **Subscription & Recurring Auto-Debit Engine (Batch 2, Items 21–40)**:
  - Standalone `contracts/subscription-engine` contract crate.
  - Automated cadence epochs (daily, weekly, monthly, annual) with billing cycle limits.
  - Prorated plan tier upgrade calculations and merchant MRR telemetry metrics.

- **Soroban Token Handling & Protocol Treasury (Batch 1, Items 1–20)**:
  - SEP-41 token interactions with allowance pre-check validation.
  - 500 bps fee ceiling with treasury destination routing.
  - Storage key isolation, bitflag reentrancy guards, and TTL auto-bump extensions.

## [1.0.0] - 2026-10-29

### Added
- Initial core release of `settlement-router` and `payment-registry`.
- Single and multi-recipient payment records on Soroban testnet.
