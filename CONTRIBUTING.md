# Contributing to Stellar Payment Hub Smart Contracts

Thank you for your interest in contributing to the **Stellar Payment Hub** Soroban smart contract suite!

## Prerequisites

- **Rust**: 1.80.0+ (`rustup default stable`)
- **WebAssembly Target**: `rustup target add wasm32-unknown-unknown`
- **Soroban CLI**: `cargo install --locked soroban-cli` (v22.0.0+)
- **Clippy & Rustfmt**: `rustup component add clippy rustfmt`

## Repository Structure

```
contracts/
├── settlement-router/     # Multi-party splits, SEP-41 token router, dynamic fee tiers
├── payment-registry/      # Stateful payment ledger, audit trail, sanctions blacklist
├── subscription-engine/   # Autonomous recurring debit epochs, cadence & MRR metrics
├── escrow-vault/          # 2-of-3 multi-sig arbitration & milestone deliverables
├── payment-stream/        # Real-time per-second linear vesting & stream payroll
└── htlc-merkle/           # Hash time-locked contracts & Merkle batch airdrops
```

## Development Workflow

1. Create a descriptive feature branch:
   ```bash
   git checkout -b feat/my-soroban-improvement
   ```
2. Build and verify all 6 workspace crates:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```
3. Run the complete Rust unit and integration test suite:
   ```bash
   cargo test --all
   ```
4. Verify code formatting and linting:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets -- -D warnings
   ```
5. Commit using [Conventional Commits](https://www.conventionalcommits.org):
   - `feat(contracts): add new capability`
   - `fix(contracts): resolve issue`
   - `test(contracts): add unit or invariant test`

## Smart Contract Guidelines

- All contract crates must declare `#![no_std]` in their root `src/lib.rs`.
- Prevent arithmetic overflows using checked math (`checked_add`, `checked_sub`, `checked_mul`, `checked_div`).
- All token operations must interface via `soroban_sdk::token::Client` with explicit allowance checks.
- Adhere strictly to the Soroban 32-character export function name limit.
- Ensure TTL auto-extensions (`extend_ttl`) are configured for long-term ledger entry persistence.
