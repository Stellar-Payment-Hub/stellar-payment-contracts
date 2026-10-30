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

    pub fn status(_env: Env) -> Symbol {
        symbol_short!("htlc_ok")
    }
}
