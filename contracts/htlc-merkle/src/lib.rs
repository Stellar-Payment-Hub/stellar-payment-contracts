pub mod errors;
pub mod events;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

use errors::HtlcError;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Bytes, BytesN, Env, Symbol, Vec};
use types::{HtlcRecord, HtlcStatus, MerkleDistribution};

#[contract]
pub struct HtlcMerkleVault;

#[contractimpl]
impl HtlcMerkleVault {
    /// Initialize contract with admin authority.
    pub fn initialize(env: Env, admin: Address) -> Result<(), HtlcError> {
        if storage::get_admin(&env).is_some() {
            return Err(HtlcError::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        env.storage().instance().extend_ttl(50_000, 100_000);
        Ok(())
    }

    /// Health check status.
    /// Create a Hash Time-Locked Contract locking tokens under a SHA-256 hashlock.
    pub fn create_htlc(
        env: Env,
        sender: Address,
        recipient: Address,
        token: Address,
        amount: i128,
        hashlock: BytesN<32>,
        duration_seconds: u64,
    ) -> Result<u64, HtlcError> {
        sender.require_auth();
        if amount <= 0 {
            return Err(HtlcError::InvalidAmount);
        }
        if sender == recipient {
            return Err(HtlcError::SamePartyAddress);
        }
        if duration_seconds < 3600 { // Minimum 1 hour timelock
            return Err(HtlcError::InvalidTimelock);
        }

        let now = env.ledger().timestamp();
        let timelock = now + duration_seconds;
        let id = storage::increment_htlc_count(&env);

        let record = HtlcRecord {
            id,
            sender: sender.clone(),
            recipient: recipient.clone(),
            token,
            amount,
            hashlock,
            timelock,
            status: HtlcStatus::Active,
            preimage: None,
            created_at: now,
        };
        storage::set_htlc(&env, id, &record);
        events::emit_htlc_created(&env, id, &sender, &recipient, amount);
        storage::extend_htlc_ttl(&env, id);
        Ok(id)
    }

    /// Query HTLC swap details by ID.
    pub fn get_htlc(env: Env, id: u64) -> Option<HtlcRecord> {
        storage::get_htlc(&env, id)
    }

    /// Claim HTLC tokens by providing the 32-byte SHA-256 preimage before timelock expiry.
    pub fn claim_htlc(
        env: Env,
        recipient: Address,
        id: u64,
        preimage: BytesN<32>,
    ) -> Result<i128, HtlcError> {
        recipient.require_auth();
        let mut record = storage::get_htlc(&env, id).ok_or(HtlcError::HtlcNotFound)?;
        if record.recipient != recipient {
            return Err(HtlcError::Unauthorized);
        }
        if record.status != HtlcStatus::Active {
            return Err(HtlcError::HtlcNotActive);
        }

        let now = env.ledger().timestamp();
        if now >= record.timelock {
            return Err(HtlcError::TimelockExpired);
        }

        // Compute SHA-256 of candidate preimage using Soroban crypto
        let mut preimage_bytes = Bytes::new(&env);
        preimage_bytes.append(&preimage.clone().into());
        let calculated_hash = env.crypto().sha256(&preimage_bytes);

        if calculated_hash != record.hashlock {
            return Err(HtlcError::InvalidPreimage);
        }

        record.status = HtlcStatus::Claimed;
        record.preimage = Some(preimage.clone());
        storage::set_htlc(&env, id, &record);
        events::emit_htlc_claimed(&env, id, &recipient, &preimage);
        storage::extend_htlc_ttl(&env, id);
        Ok(record.amount)
    }

    /// Refund HTLC tokens back to sender after timelock has expired without claim.
    pub fn refund_htlc(env: Env, sender: Address, id: u64) -> Result<i128, HtlcError> {
        sender.require_auth();
        let mut record = storage::get_htlc(&env, id).ok_or(HtlcError::HtlcNotFound)?;
        if record.sender != sender {
            return Err(HtlcError::Unauthorized);
        }
        if record.status != HtlcStatus::Active {
            return Err(HtlcError::HtlcNotActive);
        }

        let now = env.ledger().timestamp();
        if now < record.timelock {
            return Err(HtlcError::TimelockNotExpired);
        }

        record.status = HtlcStatus::Refunded;
        storage::set_htlc(&env, id, &record);
        events::emit_htlc_refunded(&env, id, &sender);
        storage::extend_htlc_ttl(&env, id);
        Ok(record.amount)
    }

    /// Public utility: compute SHA-256 hash of a 32-byte secret.
    pub fn hash_preimage(env: Env, preimage: BytesN<32>) -> BytesN<32> {
        let mut b = Bytes::new(&env);
        b.append(&preimage.into());
        env.crypto().sha256(&b)
    }

    /// Register a compressed Merkle distribution root for batch disbursements.
    pub fn register_merkle_dist(
        env: Env,
        distributor: Address,
        token: Address,
        root: BytesN<32>,
        total_amount: i128,
        duration_seconds: u64,
    ) -> Result<u64, HtlcError> {
        distributor.require_auth();
        if total_amount <= 0 {
            return Err(HtlcError::InvalidAmount);
        }
        let now = env.ledger().timestamp();
        let expires_at = now + duration_seconds;
        let id = storage::increment_dist_count(&env);

        let dist = MerkleDistribution {
            id,
            distributor,
            token,
            root,
            total_amount,
            remaining_amount: total_amount,
            expires_at,
            claims_count: 0,
        };
        storage::set_dist(&env, id, &dist);
        env.storage().persistent().extend_ttl(&storage::DataKey::Distribution(id), 50_000, 100_000);
        Ok(id)
    }

    pub fn status(_env: Env) -> Symbol {
        symbol_short!("htlc_ok")
    }
}
