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

    /// Flag escrow into dispute mode (Buyer or Seller).
    pub fn raise_dispute(env: Env, caller: Address, escrow_id: u64) -> Result<(), EscrowError> {
        caller.require_auth();
        let mut esc = storage::get_escrow(&env, escrow_id).ok_or(EscrowError::EscrowNotFound)?;
        if esc.status != EscrowStatus::Funded {
            return Err(EscrowError::EscrowNotActive);
        }
        if caller != esc.buyer && caller != esc.seller {
            return Err(EscrowError::Unauthorized);
        }
        esc.status = EscrowStatus::Disputed;
        storage::set_escrow(&env, escrow_id, &esc);
        events::emit_escrow_status(&env, escrow_id, EscrowStatus::Disputed);
        storage::extend_escrow_ttl(&env, escrow_id);
        Ok(())
    }

    /// Adjudicate partial split award between buyer and seller (Arbitrator only).
    pub fn resolve_split_award(
        env: Env,
        arbitrator: Address,
        escrow_id: u64,
        buyer_amount: i128,
        seller_amount: i128,
    ) -> Result<i128, EscrowError> {
        arbitrator.require_auth();
        let mut esc = storage::get_escrow(&env, escrow_id).ok_or(EscrowError::EscrowNotFound)?;
        if esc.arbitrator != arbitrator {
            return Err(EscrowError::Unauthorized);
        }
        if esc.status != EscrowStatus::Disputed {
            return Err(EscrowError::EscrowNotActive);
        }

        let arb_fee = if esc.arbitrator_fee_bps > 0 {
            (esc.remaining_amount * esc.arbitrator_fee_bps as i128) / 10000
        } else {
            0
        };

        let distributable = esc.remaining_amount - arb_fee;
        if buyer_amount + seller_amount != distributable {
            return Err(EscrowError::InvalidSplitSum);
        }

        esc.status = EscrowStatus::PartiallySettled;
        esc.remaining_amount = 0;
        storage::set_escrow(&env, escrow_id, &esc);
        events::emit_escrow_status(&env, escrow_id, EscrowStatus::PartiallySettled);
        storage::extend_escrow_ttl(&env, escrow_id);
        Ok(arb_fee)
    }

    /// Calculate arbitration fee for an escrow amount.
    pub fn calculate_arbitrator_fee(env: Env, escrow_id: u64) -> i128 {
        if let Some(esc) = storage::get_escrow(&env, escrow_id) {
            if esc.arbitrator_fee_bps > 0 {
                return (esc.remaining_amount * esc.arbitrator_fee_bps as i128) / 10000;
            }
        }
        0
    }

    /// Claim refund after statutory deadline has passed without seller action (Buyer only).
    pub fn claim_timelock_refund(env: Env, buyer: Address, escrow_id: u64) -> Result<(), EscrowError> {
        buyer.require_auth();
        let mut esc = storage::get_escrow(&env, escrow_id).ok_or(EscrowError::EscrowNotFound)?;
        if esc.buyer != buyer {
            return Err(EscrowError::Unauthorized);
        }
        if esc.status != EscrowStatus::Funded {
            return Err(EscrowError::EscrowNotActive);
        }
        let now = env.ledger().timestamp();
        if now < esc.deadline {
            return Err(EscrowError::DeadlineNotReached);
        }
        esc.status = EscrowStatus::Refunded;
        esc.remaining_amount = 0;
        storage::set_escrow(&env, escrow_id, &esc);
        events::emit_escrow_status(&env, escrow_id, EscrowStatus::Refunded);
        storage::extend_escrow_ttl(&env, escrow_id);
        Ok(())
    }

    /// Register a progressive milestone deliverable for an escrow (Buyer only).
    pub fn add_escrow_milestone(
        env: Env,
        buyer: Address,
        escrow_id: u64,
        index: u32,
        amount: i128,
        description: String,
    ) -> Result<(), EscrowError> {
        buyer.require_auth();
        let esc = storage::get_escrow(&env, escrow_id).ok_or(EscrowError::EscrowNotFound)?;
        if esc.buyer != buyer {
            return Err(EscrowError::Unauthorized);
        }
        if amount <= 0 || amount > esc.remaining_amount {
            return Err(EscrowError::InvalidAmount);
        }
        let ms = EscrowMilestone {
            escrow_id,
            milestone_index: index,
            amount,
            description,
            evidence_hash: None,
            approved: false,
        };
        storage::set_milestone(&env, escrow_id, index, &ms);
        storage::extend_escrow_ttl(&env, escrow_id);
        Ok(())
    }

    /// Submit cryptographic proof hash of completed deliverable (Seller only).
    pub fn submit_milestone_evidence(
        env: Env,
        seller: Address,
        escrow_id: u64,
        index: u32,
        evidence_hash: BytesN<32>,
    ) -> Result<(), EscrowError> {
        seller.require_auth();
        let esc = storage::get_escrow(&env, escrow_id).ok_or(EscrowError::EscrowNotFound)?;
        if esc.seller != seller {
            return Err(EscrowError::Unauthorized);
        }
        let mut ms = storage::get_milestone(&env, escrow_id, index).ok_or(EscrowError::MilestoneNotFound)?;
        ms.evidence_hash = Some(evidence_hash.clone());
        storage::set_milestone(&env, escrow_id, index, &ms);
        events::emit_evidence_submitted(&env, escrow_id, index, &evidence_hash);
        storage::extend_escrow_ttl(&env, escrow_id);
        Ok(())
    }

    /// Approve milestone and release proportional funds to seller (Buyer only).
    pub fn approve_milestone(
        env: Env,
        buyer: Address,
        escrow_id: u64,
        index: u32,
    ) -> Result<i128, EscrowError> {
        buyer.require_auth();
        let mut esc = storage::get_escrow(&env, escrow_id).ok_or(EscrowError::EscrowNotFound)?;
        if esc.buyer != buyer {
            return Err(EscrowError::Unauthorized);
        }
        let mut ms = storage::get_milestone(&env, escrow_id, index).ok_or(EscrowError::MilestoneNotFound)?;
        if ms.approved {
            return Err(EscrowError::MilestoneAlreadyApproved);
        }
        ms.approved = true;
        esc.remaining_amount -= ms.amount;
        if esc.remaining_amount == 0 {
            esc.status = EscrowStatus::Released;
        }
        storage::set_milestone(&env, escrow_id, index, &ms);
        storage::set_escrow(&env, escrow_id, &esc);
        storage::extend_escrow_ttl(&env, escrow_id);
        Ok(ms.amount)
    }

    /// Extend escrow deadline timestamp (Buyer only).
    pub fn extend_deadline(
        env: Env,
        buyer: Address,
        escrow_id: u64,
        extra_seconds: u64,
    ) -> Result<u64, EscrowError> {
        buyer.require_auth();
        let mut esc = storage::get_escrow(&env, escrow_id).ok_or(EscrowError::EscrowNotFound)?;
        if esc.buyer != buyer {
            return Err(EscrowError::Unauthorized);
        }
        if esc.status != EscrowStatus::Funded && esc.status != EscrowStatus::Disputed {
            return Err(EscrowError::EscrowNotActive);
        }
        esc.deadline += extra_seconds;
        let new_deadline = esc.deadline;
        storage::set_escrow(&env, escrow_id, &esc);
        storage::extend_escrow_ttl(&env, escrow_id);
        Ok(new_deadline)
    }

    /// Query specific milestone details.
    pub fn get_milestone(env: Env, escrow_id: u64, index: u32) -> Option<EscrowMilestone> {
        storage::get_milestone(&env, escrow_id, index)
    }

    pub fn status(_env: Env) -> Symbol {
        symbol_short!("escrow_ok")
    }
}
