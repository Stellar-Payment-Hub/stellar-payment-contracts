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
        let calculated_hash: BytesN<32> = env.crypto().sha256(&preimage_bytes).into();

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
        env.crypto().sha256(&b).into()
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

    /// Query Merkle distribution parameters by ID.
    pub fn get_merkle_dist(env: Env, id: u64) -> Option<MerkleDistribution> {
        storage::get_dist(&env, id)
    }

    /// Check if a specific leaf index has already been claimed.
    pub fn is_leaf_claimed(env: Env, dist_id: u64, index: u32) -> bool {
        storage::is_leaf_claimed(&env, dist_id, index)
    }

    /// Claim tokens from Merkle distribution using cryptographic proof.
    pub fn claim_merkle_leaf(
        env: Env,
        recipient: Address,
        dist_id: u64,
        index: u32,
        amount: i128,
        proof: Vec<BytesN<32>>,
    ) -> Result<i128, HtlcError> {
        recipient.require_auth();
        let mut dist = storage::get_dist(&env, dist_id).ok_or(HtlcError::DistributionNotFound)?;
        let now = env.ledger().timestamp();
        if now >= dist.expires_at {
            return Err(HtlcError::DistributionExpired);
        }
        if storage::is_leaf_claimed(&env, dist_id, index) {
            return Err(HtlcError::LeafAlreadyClaimed);
        }

        // Compute leaf hash = sha256(index || amount)
        let mut leaf_bytes = Bytes::new(&env);
        leaf_bytes.append(&Bytes::from_slice(&env, &index.to_be_bytes()));
        leaf_bytes.append(&Bytes::from_slice(&env, &amount.to_be_bytes()));
        let mut current_hash: BytesN<32> = env.crypto().sha256(&leaf_bytes).into();

        // Verify merkle proof
        let proof_len = proof.len();
        for i in 0..proof_len {
            let sibling = proof.get(i).unwrap();
            let mut combined = Bytes::new(&env);
            if current_hash < sibling {
                combined.append(&current_hash.into());
                combined.append(&sibling.into());
            } else {
                combined.append(&sibling.into());
                combined.append(&current_hash.into());
            }
            current_hash = env.crypto().sha256(&combined).into();
        }

        if current_hash != dist.root {
            return Err(HtlcError::InvalidProof);
        }

        storage::set_leaf_claimed(&env, dist_id, index);
        dist.remaining_amount -= amount;
        dist.claims_count += 1;
        storage::set_dist(&env, dist_id, &dist);
        events::emit_merkle_claimed(&env, dist_id, index, &recipient, amount);
        Ok(amount)
    }

    /// Reclaim unspent tokens after Merkle distribution expiration (Distributor only).
    pub fn reclaim_unspent_merkle(env: Env, distributor: Address, dist_id: u64) -> Result<i128, HtlcError> {
        distributor.require_auth();
        let mut dist = storage::get_dist(&env, dist_id).ok_or(HtlcError::DistributionNotFound)?;
        if dist.distributor != distributor {
            return Err(HtlcError::Unauthorized);
        }
        let now = env.ledger().timestamp();
        if now < dist.expires_at {
            return Err(HtlcError::DistributionNotExpired);
        }
        let unspent = dist.remaining_amount;
        dist.remaining_amount = 0;
        storage::set_dist(&env, dist_id, &dist);
        Ok(unspent)
    }

    /// Query remaining seconds until HTLC timelock expires.
    pub fn get_htlc_time_remaining(env: Env, id: u64) -> u64 {
        if let Some(h) = storage::get_htlc(&env, id) {
            let now = env.ledger().timestamp();
            if h.timelock > now {
                return h.timelock - now;
            }
        }
        0
    }

    /// Manually renew storage TTL for an active HTLC swap.
    pub fn extend_swap_ttl(env: Env, id: u64) -> Result<(), HtlcError> {
        let _rec = storage::get_htlc(&env, id).ok_or(HtlcError::HtlcNotFound)?;
        storage::extend_htlc_ttl(&env, id);
        Ok(())
    }

    /// Manually renew storage TTL for an active Merkle distribution.
    pub fn extend_dist_ttl(env: Env, id: u64) -> Result<(), HtlcError> {
        let _dist = storage::get_dist(&env, id).ok_or(HtlcError::DistributionNotFound)?;
        env.storage().persistent().extend_ttl(&storage::DataKey::Distribution(id), 50_000, 100_000);
        Ok(())
    }

    pub fn status(_env: Env) -> Symbol {
        symbol_short!("htlc_ok")
    }
}
