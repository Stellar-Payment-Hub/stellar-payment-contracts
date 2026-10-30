#![no_std]
pub mod errors;
pub mod events;
pub mod storage;
pub mod types;
pub mod token_vault;

#[cfg(test)]
mod test;

use errors::SettlementError;
use soroban_sdk::{
    contract, contractclient, contractimpl, symbol_short, Address, Env, String, Symbol, Vec,
};
use types::{DisputeVerdict, EscrowTimelock, FeeConfig, RecipientShare, SettlementDispute, SettlementMilestone, SettlementRecord, SettlementStatus, WeightedShare};
use soroban_sdk::BytesN;

/// Client interface for Soroban Inter-Contract Communication
/// Calling the deployed PaymentRegistry contract.
#[contractclient(name = "PaymentRegistryClient")]
pub trait PaymentRegistryInterface {
    fn create_payment(
        env: Env,
        creator: Address,
        recipient: Address,
        amount: i128,
        memo: String,
    ) -> u64;
}

#[contract]
pub struct SettlementRouter;

impl SettlementRouter {
    fn create_settlement_internal(
        env: &Env,
        payer: &Address,
        recipients: &Vec<RecipientShare>,
        memo: &String,
    ) -> Result<u64, SettlementError> {
        let count = recipients.len();
        if count == 0 {
            return Err(SettlementError::EmptyRecipients);
        }
        if count > 100 {
            return Err(SettlementError::TooManyRecipients);
        }

        let mut calculated_total: i128 = 0;
        for i in 0..count {
            let share = recipients.get(i).unwrap();
            if share.amount <= 0 {
                return Err(SettlementError::InvalidShareAmount);
            }
            if &share.recipient == payer {
                return Err(SettlementError::PayerIsRecipient);
            }
            calculated_total += share.amount;
        }

        let min_thresh = storage::get_min_threshold(env);
        if calculated_total < min_thresh {
            return Err(SettlementError::InvalidTotalAmount);
        }

        let settlement_id = storage::increment_counter(env);
        let timestamp = env.ledger().timestamp();

        soroban_sdk::log!(env, "Creating settlement with recipient count: {}", count as u32);
        storage::extend_instance_ttl(env);
        let record = SettlementRecord {
            id: settlement_id,
            payer: payer.clone(),
            token: None,
            total_amount: calculated_total,
            fee_amount: 0,
            recipient_count: count as u32,
            memo: memo.clone(),
            memo_hash: None,
            status: SettlementStatus::Created,
            sub_payment_ids: Vec::new(env),
            created_at: timestamp,
            updated_at: timestamp,
        };

        storage::set_settlement(env, settlement_id, &record);
        storage::set_recipients(env, settlement_id, recipients);

        events::emit_settlement_created(
            env,
            settlement_id,
            payer,
            calculated_total,
            count as u32,
        );

        Ok(settlement_id)
    }
}

#[contractimpl]
impl SettlementRouter {
    /// Initialize the SettlementRouter with an admin and registry contract address.
    pub fn initialize(
        env: Env,
        admin: Address,
        registry_contract: Address,
    ) -> Result<(), SettlementError> {
        if storage::get_admin(&env).is_some() {
            return Err(SettlementError::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        storage::set_registry(&env, &registry_contract);
        storage::extend_instance_ttl(&env);
        Ok(())
    }

    /// Configure protocol treasury address (Admin only).
    /// Whitelist or configure a Soroban token for settlements (Admin only).
    pub fn set_token_whitelist(
        env: Env,
        admin: Address,
        token: Address,
        enabled: bool,
        min_deposit: i128,
        max_deposit: i128,
        decimals: u32,
    ) -> Result<(), SettlementError> {
        admin.require_auth();
        let current_admin = storage::get_admin(&env).ok_or(SettlementError::Unauthorized)?;
        if admin != current_admin {
            return Err(SettlementError::Unauthorized);
        }
        let config = types::TokenConfig {
            address: token.clone(),
            min_deposit,
            max_deposit,
            enabled,
            decimals,
        };
        storage::set_token_whitelist(&env, &token, &config);
        events::emit_token_whitelist_updated(&env, &token, enabled, min_deposit);
        storage::extend_instance_ttl(&env);
        Ok(())
    }

    pub fn set_treasury(env: Env, admin: Address, treasury: Address) -> Result<(), SettlementError> {
        admin.require_auth();
        let current_admin = storage::get_admin(&env).ok_or(SettlementError::Unauthorized)?;
        if admin != current_admin {
            return Err(SettlementError::Unauthorized);
        }
        storage::set_treasury(&env, &treasury);
        storage::extend_instance_ttl(&env);
        Ok(())
    }

    /// Configure protocol fee parameters in basis points (Admin only).
    pub fn set_fee_config(
        env: Env,
        admin: Address,
        fee_bps: u32,
        max_fee_bps: u32,
        enabled: bool,
    ) -> Result<(), SettlementError> {
        admin.require_auth();
        let current_admin = storage::get_admin(&env).ok_or(SettlementError::Unauthorized)?;
        if admin != current_admin {
            return Err(SettlementError::Unauthorized);
        }
        if fee_bps > max_fee_bps || max_fee_bps > 1000 { // Max 10%
            return Err(SettlementError::InvalidFeeBps);
        }
        let config = FeeConfig { fee_bps, max_fee_bps, enabled };
        storage::set_fee_config(&env, &config);
        storage::extend_instance_ttl(&env);
        Ok(())
    }

    /// Query current fee configuration.
    pub fn get_fee_config(env: Env) -> Option<FeeConfig> {
        storage::get_fee_config(&env)
    }

    /// Query current treasury address.
    pub fn get_treasury(env: Env) -> Option<Address> {
        storage::get_treasury(&env)
    }

    /// Read contract health/status symbol.
    pub fn status(_env: Env) -> Symbol {
        symbol_short!("settle_ok")
    }

    /// Read settlement count in storage.
    pub fn get_settlement_count(env: Env) -> u64 {
        storage::get_counter(&env)
    }

    /// Set designated arbitrator address (Admin only).
    pub fn set_arbitrator(env: Env, admin: Address, arbitrator: Address) -> Result<(), SettlementError> {
        admin.require_auth();
        let current_admin = storage::get_admin(&env).ok_or(SettlementError::Unauthorized)?;
        if admin != current_admin {
            return Err(SettlementError::Unauthorized);
        }
        storage::set_arbitrator(&env, &arbitrator);
        storage::extend_instance_ttl(&env);
        Ok(())
    }

    /// Read designated arbitrator address.
    pub fn get_arbitrator(env: Env) -> Option<Address> {
        storage::get_arbitrator(&env)
    }

    /// Raise a dispute against an active settlement with cryptographic evidence hash.
    pub fn raise_dispute(
        env: Env,
        caller: Address,
        settlement_id: u64,
        evidence_hash: BytesN<32>,
    ) -> Result<(), SettlementError> {
        caller.require_auth();

        let record = storage::get_settlement(&env, settlement_id)
            .ok_or(SettlementError::SettlementNotFound)?;

        if record.payer != caller {
            return Err(SettlementError::Unauthorized);
        }
        if record.status != SettlementStatus::Created {
            return Err(SettlementError::InvalidStatusTransition);
        }
        if storage::get_settlement_dispute(&env, settlement_id).is_some() {
            return Err(SettlementError::DisputeActive);
        }

        let now = env.ledger().timestamp();
        let dispute = SettlementDispute {
            settlement_id,
            initiator: caller.clone(),
            evidence_hash: evidence_hash.clone(),
            verdict: DisputeVerdict::Pending,
            initiated_at: now,
            resolved_at: None,
        };

        storage::set_settlement_dispute(&env, settlement_id, &dispute);
        storage::extend_settlement_ttl(&env, settlement_id);
        events::emit_dispute_raised(&env, settlement_id, &caller, &evidence_hash);
        Ok(())
    }

    /// Adjudicate and resolve a disputed settlement (Arbitrator only).
    pub fn resolve_dispute(
        env: Env,
        caller: Address,
        settlement_id: u64,
        verdict: DisputeVerdict,
    ) -> Result<(), SettlementError> {
        caller.require_auth();

        let arbitrator = storage::get_arbitrator(&env).ok_or(SettlementError::Unauthorized)?;
        if caller != arbitrator {
            return Err(SettlementError::Unauthorized);
        }

        let mut dispute = storage::get_settlement_dispute(&env, settlement_id)
            .ok_or(SettlementError::SettlementNotFound)?;

        let mut record = storage::get_settlement(&env, settlement_id)
            .ok_or(SettlementError::SettlementNotFound)?;

        let now = env.ledger().timestamp();
        dispute.verdict = verdict;
        dispute.resolved_at = Some(now);
        storage::set_settlement_dispute(&env, settlement_id, &dispute);

        match verdict {
            DisputeVerdict::RefundPayer => {
                record.status = SettlementStatus::Cancelled;
                record.updated_at = now;
                storage::set_settlement(&env, settlement_id, &record);
            }
            _ => {
                record.updated_at = now;
                storage::set_settlement(&env, settlement_id, &record);
            }
        }

        events::emit_dispute_resolved(&env, settlement_id, verdict as u32);
        storage::extend_settlement_ttl(&env, settlement_id);
        Ok(())
    }

    /// Query dispute record for a settlement.
    pub fn get_dispute(env: Env, settlement_id: u64) -> Result<SettlementDispute, SettlementError> {
        storage::get_settlement_dispute(&env, settlement_id).ok_or(SettlementError::SettlementNotFound)
    }

    /// Attach a conditional timelock to a created settlement.
    pub fn attach_timelock(
        env: Env,
        caller: Address,
        settlement_id: u64,
        release_after: u64,
        auto_refund_after: u64,
    ) -> Result<(), SettlementError> {
        caller.require_auth();

        let record = storage::get_settlement(&env, settlement_id)
            .ok_or(SettlementError::SettlementNotFound)?;

        if record.payer != caller {
            return Err(SettlementError::Unauthorized);
        }
        if record.status != SettlementStatus::Created {
            return Err(SettlementError::InvalidStatusTransition);
        }

        let now = env.ledger().timestamp();
        if release_after < now || auto_refund_after <= release_after {
            return Err(SettlementError::DeadlineExpired);
        }

        let timelock = EscrowTimelock { release_after, auto_refund_after };
        storage::set_timelock(&env, settlement_id, &timelock);
        storage::extend_settlement_ttl(&env, settlement_id);
        Ok(())
    }

    /// Read timelock parameters for a settlement.
    pub fn get_timelock(env: Env, settlement_id: u64) -> Result<EscrowTimelock, SettlementError> {
        storage::get_timelock(&env, settlement_id).ok_or(SettlementError::SettlementNotFound)
    }

    /// Set minimum settlement threshold (Admin only).
    pub fn set_min_threshold(env: Env, admin: Address, min: i128) -> Result<(), SettlementError> {
        admin.require_auth();
        let current_admin = storage::get_admin(&env).ok_or(SettlementError::Unauthorized)?;
        if admin != current_admin {
            return Err(SettlementError::Unauthorized);
        }
        if min <= 0 {
            return Err(SettlementError::InvalidTotalAmount);
        }
        storage::set_min_threshold(&env, min);
        storage::extend_instance_ttl(&env);
        Ok(())
    }

    /// Read minimum settlement threshold.
    pub fn get_min_threshold(env: Env) -> i128 {
        storage::get_min_threshold(&env)
    }

    /// Read lifetime settled volume across all completed settlements.
    pub fn get_total_volume(env: Env) -> i128 {
        storage::get_total_volume(&env)
    }

    /// Register a project milestone for progressive disbursement.
    pub fn add_milestone(
        env: Env,
        caller: Address,
        settlement_id: u64,
        index: u32,
        description: String,
        amount: i128,
    ) -> Result<(), SettlementError> {
        caller.require_auth();

        let record = storage::get_settlement(&env, settlement_id)
            .ok_or(SettlementError::SettlementNotFound)?;

        if record.payer != caller {
            return Err(SettlementError::Unauthorized);
        }
        if amount <= 0 || amount > record.total_amount {
            return Err(SettlementError::InvalidShareAmount);
        }

        let milestone = SettlementMilestone {
            settlement_id,
            milestone_index: index,
            description,
            amount,
            executed: false,
        };

        storage::set_milestone(&env, settlement_id, index, &milestone);
        storage::extend_settlement_ttl(&env, settlement_id);
        Ok(())
    }

    /// Read milestone details.
    pub fn get_milestone(env: Env, settlement_id: u64, index: u32) -> Result<SettlementMilestone, SettlementError> {
        storage::get_milestone(&env, settlement_id, index).ok_or(SettlementError::MilestoneNotFound)
    }

    /// Release milestone funds upon deliverable completion (Payer or Arbitrator).
    pub fn release_milestone(
        env: Env,
        caller: Address,
        settlement_id: u64,
        index: u32,
    ) -> Result<(), SettlementError> {
        caller.require_auth();

        let record = storage::get_settlement(&env, settlement_id)
            .ok_or(SettlementError::SettlementNotFound)?;

        let arbitrator = storage::get_arbitrator(&env);
        let is_payer = record.payer == caller;
        let is_arb = arbitrator.map(|a| a == caller).unwrap_or(false);

        if !is_payer && !is_arb {
            return Err(SettlementError::Unauthorized);
        }

        let mut milestone = storage::get_milestone(&env, settlement_id, index)
            .ok_or(SettlementError::MilestoneNotFound)?;

        if milestone.executed {
            return Err(SettlementError::MilestoneAlreadyExecuted);
        }

        milestone.executed = true;
        storage::set_milestone(&env, settlement_id, index, &milestone);
        events::emit_milestone_released(&env, settlement_id, index, milestone.amount);
        storage::extend_settlement_ttl(&env, settlement_id);
        Ok(())
    }

    /// Pause router operations (Admin only).
    pub fn pause(env: Env, admin: Address) -> Result<(), SettlementError> {
        admin.require_auth();
        let current_admin = storage::get_admin(&env).ok_or(SettlementError::Unauthorized)?;
        if admin != current_admin {
            return Err(SettlementError::Unauthorized);
        }
        storage::set_paused(&env, true);
        events::emit_router_paused(&env, &admin, true);
        storage::extend_instance_ttl(&env);
        Ok(())
    }

    /// Unpause router operations (Admin only).
    pub fn unpause(env: Env, admin: Address) -> Result<(), SettlementError> {
        admin.require_auth();
        let current_admin = storage::get_admin(&env).ok_or(SettlementError::Unauthorized)?;
        if admin != current_admin {
            return Err(SettlementError::Unauthorized);
        }
        storage::set_paused(&env, false);
        events::emit_router_paused(&env, &admin, false);
        storage::extend_instance_ttl(&env);
        Ok(())
    }

    /// Check if router is currently paused.
    /// Check if a token is whitelisted for settlements.
    pub fn is_token_whitelisted(env: Env, token: Address) -> bool {
        storage::is_token_whitelisted(&env, &token)
    }

    /// Query full token configuration parameters.
    pub fn get_token_config(env: Env, token: Address) -> Option<types::TokenConfig> {
        storage::get_token_config(&env, &token)
    }

    pub fn is_paused(env: Env) -> bool {
        storage::is_paused(&env)
    }

    /// Upgrade router contract code to new WASM hash (Admin only).
    pub fn upgrade(env: Env, admin: Address, new_wasm_hash: BytesN<32>) -> Result<(), SettlementError> {
        admin.require_auth();
        let current_admin = storage::get_admin(&env).ok_or(SettlementError::Unauthorized)?;
        if admin != current_admin {
            return Err(SettlementError::Unauthorized);
        }
        env.deployer().update_current_contract_wasm(new_wasm_hash.clone());
        events::emit_router_upgraded(&env, &admin, &new_wasm_hash);
        Ok(())
    }

    /// Read the associated PaymentRegistry contract address.
    pub fn get_registry(env: Env) -> Result<Address, SettlementError> {
        storage::get_registry(&env).ok_or(SettlementError::NotInitialized)
    }

    /// Create a multi-recipient payment settlement request.
    pub fn create_settlement(
        env: Env,
        payer: Address,
        recipients: Vec<RecipientShare>,
        memo: String,
    ) -> Result<u64, SettlementError> {
        payer.require_auth();
        Self::create_settlement_internal(&env, &payer, &recipients, &memo)
    }

    /// Execute settlement: Invokes the PaymentRegistry contract for each recipient
    /// (Inter-contract communication) and marks settlement completed.
    pub fn execute_settlement(
        env: Env,
        settlement_id: u64,
        caller: Address,
    ) -> Result<u64, SettlementError> {
        if storage::is_paused(&env) {
            return Err(SettlementError::ContractPaused);
        }
        if storage::is_reentrancy_locked(&env) {
            return Err(SettlementError::ReentrancyGuard);
        }
        if let Some(lock) = storage::get_timelock(&env, settlement_id) {
            let now = env.ledger().timestamp();
            if now < lock.release_after {
                return Err(SettlementError::DeadlineNotPassed);
            }
        }
        if let Some(dispute) = storage::get_settlement_dispute(&env, settlement_id) {
            if dispute.verdict == DisputeVerdict::Pending {
                return Err(SettlementError::DisputeActive);
            }
        }
        storage::set_reentrancy_lock(&env, true);
        caller.require_auth();

        let mut record = storage::get_settlement(&env, settlement_id)
            .ok_or(SettlementError::SettlementNotFound)?;

        if record.payer != caller {
            return Err(SettlementError::Unauthorized);
        }

        if record.status != SettlementStatus::Created && record.status != SettlementStatus::Processing {
            return Err(SettlementError::InvalidStatusTransition);
        }

        let recipients = storage::get_recipients(&env, settlement_id)
            .ok_or(SettlementError::SettlementNotFound)?;

        let registry_addr = storage::get_registry(&env)
            .ok_or(SettlementError::NotInitialized)?;

        let registry_client = PaymentRegistryClient::new(&env, &registry_addr);

        let mut sub_ids = Vec::new(&env);
        for i in 0..recipients.len() {
            let share = recipients.get(i).unwrap();

            // Inter-contract invocation: calling PaymentRegistry::create_payment
            let sub_id = registry_client.create_payment(
                &record.payer,
                &share.recipient,
                &share.amount,
                &record.memo,
            );

            sub_ids.push_back(sub_id);

            events::emit_recipient_processed(
                &env,
                settlement_id,
                &share.recipient,
                share.amount,
                sub_id,
            );
        }

        record.status = SettlementStatus::Completed;
        record.sub_payment_ids = sub_ids;
        record.updated_at = env.ledger().timestamp();

        storage::set_settlement(&env, settlement_id, &record);

        storage::add_settled_volume(&env, record.total_amount);
        events::emit_settlement_completed(
            &env,
            settlement_id,
            &record.payer,
            record.total_amount,
        );

        storage::set_reentrancy_lock(&env, false);
        storage::extend_settlement_ttl(&env, settlement_id);
        Ok(settlement_id)
    }

    /// Split a bill evenly among a list of recipients.
    pub fn split_bill(
        env: Env,
        payer: Address,
        total_amount: i128,
        recipients: Vec<Address>,
        memo: String,
    ) -> Result<u64, SettlementError> {
        payer.require_auth();

        let count = recipients.len();
        if count == 0 {
            return Err(SettlementError::EmptyRecipients);
        }
        if total_amount <= 0 {
            return Err(SettlementError::InvalidTotalAmount);
        }

        let share_amount = total_amount / (count as i128);
        if share_amount <= 0 {
            return Err(SettlementError::InvalidShareAmount);
        }

        let mut recipient_shares = Vec::new(&env);
        let mut allocated: i128 = 0;

        for i in 0..count {
            let recipient = recipients.get(i).unwrap();
            let amount = if i == count - 1 {
                // Add any division remainder to the final recipient
                total_amount - allocated
            } else {
                share_amount
            };
            allocated += amount;

            recipient_shares.push_back(RecipientShare { recipient, amount });
        }

        Self::create_settlement_internal(&env, &payer, &recipient_shares, &memo)
    }

    /// Split a bill by custom integer weights with remainder preservation to the first recipient.
    pub fn split_bill_weighted(
        env: Env,
        payer: Address,
        total_amount: i128,
        shares: Vec<WeightedShare>,
        memo: String,
    ) -> Result<u64, SettlementError> {
        payer.require_auth();

        let count = shares.len();
        if count == 0 {
            return Err(SettlementError::EmptyRecipients);
        }
        if total_amount <= 0 {
            return Err(SettlementError::InvalidTotalAmount);
        }

        let mut total_weight: u64 = 0;
        for i in 0..count {
            let share = shares.get(i).unwrap();
            if share.weight == 0 {
                return Err(SettlementError::InvalidWeight);
            }
            total_weight += share.weight as u64;
        }

        if total_weight == 0 {
            return Err(SettlementError::ZeroWeightSum);
        }

        let mut recipient_shares = Vec::new(&env);
        let mut allocated: i128 = 0;

        for i in 0..count {
            let share = shares.get(i).unwrap();
            let amount = if i == count - 1 {
                total_amount - allocated
            } else {
                (total_amount * (share.weight as i128)) / (total_weight as i128)
            };
            allocated += amount;
            recipient_shares.push_back(RecipientShare { recipient: share.recipient, amount });
        }

        Self::create_settlement_internal(&env, &payer, &recipient_shares, &memo)
    }

    /// Retrieve settlement record.
    pub fn get_settlement(env: Env, settlement_id: u64) -> Result<SettlementRecord, SettlementError> {
        storage::extend_instance_ttl(&env);
        storage::get_settlement(&env, settlement_id).ok_or(SettlementError::SettlementNotFound)
    }

    /// Read a paginated list of settlement records.
    pub fn paginate_settlements(env: Env, start_id: u64, limit: u32) -> Vec<SettlementRecord> {
        let total = storage::get_counter(&env);
        if total > 0 {
            storage::extend_instance_ttl(&env);
        }
        let mut results = Vec::new(&env);

        if start_id == 0 || start_id > total {
            return results;
        }

        let max = if (start_id + limit as u64 - 1) < total {
            start_id + limit as u64 - 1
        } else {
            total
        };

        for id in start_id..=max {
            if let Some(record) = storage::get_settlement(&env, id) {
                results.push_back(record);
            }
        }

        results
    }

    /// Retrieve recipient shares for a settlement.
    pub fn get_recipients(
        env: Env,
        settlement_id: u64,
    ) -> Result<Vec<RecipientShare>, SettlementError> {
        storage::get_recipients(&env, settlement_id).ok_or(SettlementError::SettlementNotFound)
    }

    /// Cancel a pending settlement request.
    pub fn cancel_settlement(
        env: Env,
        settlement_id: u64,
        caller: Address,
    ) -> Result<(), SettlementError> {
        caller.require_auth();

        let mut record = storage::get_settlement(&env, settlement_id)
            .ok_or(SettlementError::SettlementNotFound)?;

        if record.payer != caller {
            return Err(SettlementError::Unauthorized);
        }

        if record.status != SettlementStatus::Created {
            return Err(SettlementError::InvalidStatusTransition);
        }

        record.status = SettlementStatus::Cancelled;
        record.updated_at = env.ledger().timestamp();
        storage::set_settlement(&env, settlement_id, &record);

        events::emit_settlement_cancelled(&env, settlement_id, &record.payer);
        Ok(())
    }

    /// Refund an unexecuted or cancelled settlement to the payer.
    pub fn refund_settlement(
        env: Env,
        settlement_id: u64,
        caller: Address,
    ) -> Result<(), SettlementError> {
        caller.require_auth();

        let mut record = storage::get_settlement(&env, settlement_id)
            .ok_or(SettlementError::SettlementNotFound)?;

        if record.payer != caller {
            return Err(SettlementError::Unauthorized);
        }

        if record.status != SettlementStatus::Cancelled && record.status != SettlementStatus::Created {
            return Err(SettlementError::InvalidStatusTransition);
        }

        record.status = SettlementStatus::Cancelled;
        record.updated_at = env.ledger().timestamp();
        storage::set_settlement(&env, settlement_id, &record);
        storage::extend_settlement_ttl(&env, settlement_id);

        events::emit_settlement_refunded(&env, settlement_id, &record.payer, record.total_amount);
        Ok(())
    }
}
