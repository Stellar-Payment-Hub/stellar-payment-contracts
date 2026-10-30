pub mod errors;
pub mod events;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

use errors::EscrowError;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env, String, Symbol};
use types::{EscrowMilestone, EscrowRecord, EscrowStatus, VoteDecision};

#[contract]
pub struct EscrowVault;

#[contractimpl]
impl EscrowVault {
    /// Initialize contract with admin authority.
    pub fn initialize(env: Env, admin: Address) -> Result<(), EscrowError> {
        if storage::get_admin(&env).is_some() {
            return Err(EscrowError::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        env.storage().instance().extend_ttl(50_000, 100_000);
        Ok(())
    }

    /// Health check status.
    /// Create a new 3-party escrow agreement with lockup.
    pub fn create_escrow(
        env: Env,
        buyer: Address,
        seller: Address,
        arbitrator: Address,
        token: Address,
        total_amount: i128,
        arbitrator_fee_bps: u32,
        duration_seconds: u64,
    ) -> Result<u64, EscrowError> {
        buyer.require_auth();
        if total_amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }
        if buyer == seller || buyer == arbitrator || seller == arbitrator {
            return Err(EscrowError::SamePartyAddress);
        }
        if arbitrator_fee_bps > 500 { // Max 5% arbitration fee
            return Err(EscrowError::FeeExceedsCap);
        }
        if duration_seconds < 3600 { // Minimum 1 hour
            return Err(EscrowError::InvalidDeadline);
        }

        let id = storage::increment_escrow_count(&env);
        let now = env.ledger().timestamp();
        let deadline = now + duration_seconds;

        let record = EscrowRecord {
            id,
            buyer: buyer.clone(),
            seller: seller.clone(),
            arbitrator,
            token,
            total_amount,
            remaining_amount: total_amount,
            arbitrator_fee_bps,
            deadline,
            status: EscrowStatus::Funded,
            created_at: now,
        };
        storage::set_escrow(&env, id, &record);
        events::emit_escrow_created(&env, id, &buyer, &seller, total_amount);
        storage::extend_escrow_ttl(&env, id);
        Ok(id)
    }

    /// Query escrow terms and state by ID.
    pub fn get_escrow(env: Env, id: u64) -> Option<EscrowRecord> {
        storage::get_escrow(&env, id)
    }

    pub fn status(_env: Env) -> Symbol {
        symbol_short!("escrow_ok")
    }
}
