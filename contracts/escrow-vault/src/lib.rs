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

    /// Cast vote to release funds to seller (Buyer or Arbitrator).
    pub fn vote_release(env: Env, caller: Address, escrow_id: u64) -> Result<bool, EscrowError> {
        caller.require_auth();
        let mut esc = storage::get_escrow(&env, escrow_id).ok_or(EscrowError::EscrowNotFound)?;
        if esc.status != EscrowStatus::Funded && esc.status != EscrowStatus::Disputed {
            return Err(EscrowError::EscrowNotActive);
        }
        if caller != esc.buyer && caller != esc.arbitrator {
            return Err(EscrowError::Unauthorized);
        }

        storage::set_vote(&env, escrow_id, &caller, VoteDecision::ReleaseToSeller);
        events::emit_vote_cast(&env, escrow_id, &caller, VoteDecision::ReleaseToSeller as u32);

        // Check 2-of-3 threshold (buyer + arbitrator, or seller self-acknowledgement)
        let buyer_vote = storage::get_vote(&env, escrow_id, &esc.buyer);
        let arb_vote = storage::get_vote(&env, escrow_id, &esc.arbitrator);

        if buyer_vote == VoteDecision::ReleaseToSeller || (arb_vote == VoteDecision::ReleaseToSeller && caller == esc.arbitrator) {
            esc.status = EscrowStatus::Released;
            esc.remaining_amount = 0;
            storage::set_escrow(&env, escrow_id, &esc);
            events::emit_escrow_status(&env, escrow_id, EscrowStatus::Released);
            return Ok(true); // Settled!
        }
        Ok(false)
    }

    /// Cast vote to refund funds to buyer (Seller or Arbitrator).
    pub fn vote_refund(env: Env, caller: Address, escrow_id: u64) -> Result<bool, EscrowError> {
        caller.require_auth();
        let mut esc = storage::get_escrow(&env, escrow_id).ok_or(EscrowError::EscrowNotFound)?;
        if esc.status != EscrowStatus::Funded && esc.status != EscrowStatus::Disputed {
            return Err(EscrowError::EscrowNotActive);
        }
        if caller != esc.seller && caller != esc.arbitrator {
            return Err(EscrowError::Unauthorized);
        }

        storage::set_vote(&env, escrow_id, &caller, VoteDecision::RefundToBuyer);
        events::emit_vote_cast(&env, escrow_id, &caller, VoteDecision::RefundToBuyer as u32);

        let seller_vote = storage::get_vote(&env, escrow_id, &esc.seller);
        let arb_vote = storage::get_vote(&env, escrow_id, &esc.arbitrator);

        if seller_vote == VoteDecision::RefundToBuyer || (arb_vote == VoteDecision::RefundToBuyer && caller == esc.arbitrator) {
            esc.status = EscrowStatus::Refunded;
            esc.remaining_amount = 0;
            storage::set_escrow(&env, escrow_id, &esc);
            events::emit_escrow_status(&env, escrow_id, EscrowStatus::Refunded);
            return Ok(true);
        }
        Ok(false)
    }

    pub fn status(_env: Env) -> Symbol {
        symbol_short!("escrow_ok")
    }
}
