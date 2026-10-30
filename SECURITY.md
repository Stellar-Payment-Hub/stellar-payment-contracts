# Security Policy

## Supported Versions

| Version | Supported          | Soroban SDK Version |
| ------- | ------------------ | ------------------- |
| 1.0.x   | :white_check_mark: | v22.0.11 (Protocol 22)|

## Smart Contract Security Invariants

The smart contracts in this repository enforce critical security invariants:
1. **Zero Stroop Leakage**: Multi-recipient splits conserve exact input amounts via integer remainder distribution.
2. **Reentrancy Protection**: Bitflag storage locks prevent reentrant calls on all state-mutating functions.
3. **SEP-41 Allowance Authorization**: Token transfers verify spender allowance before executing debits.
4. **2-of-3 Multi-Sig Escrow**: Disputes require consensus between Buyer, Seller, and neutral Arbitrator.
5. **OFAC Sanctions Blacklisting**: The `payment-registry` enforces real-time address blacklist screening.
6. **Maximum Statutory Fee Caps**: Protocol fees are hard-capped at 500 basis points (5.0%).
7. **Timelock Expiration Safety**: HTLC and escrow deposits feature deterministic refund fallbacks after deadline expiration.

## Reporting a Vulnerability

If you discover any smart contract vulnerability or potential exploit vector:
1. **DO NOT** create a public issue or forum post.
2. Email full technical details to **security@stellar-payment-hub.org**.
3. Include contract bytecode, execution trace, and reproducible Rust test code if available.
4. We acknowledge reports within 24 hours and commit to transparent remediation.
