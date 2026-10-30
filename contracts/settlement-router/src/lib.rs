#![no_std]
pub mod errors;
pub mod events;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

use errors::SettlementError;
use soroban_sdk::{
    contract, contractclient, contractimpl, symbol_short, Address, Env, String, Symbol, Vec,
};
use types::{FeeConfig, RecipientShare, SettlementMilestone, SettlementRecord, SettlementStatus, WeightedShare};
use soroban_sdk::{token, BytesN};

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

        if calculated_total <= 0 {
            return Err(SettlementError::InvalidTotalAmount);
        }

        let settlement_id = storage::increment_counter(env);
        let timestamp = env.ledger().timestamp();

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

    /// Retrieve settlement record.
    pub fn get_settlement(env: Env, settlement_id: u64) -> Result<SettlementRecord, SettlementError> {
        storage::extend_instance_ttl(&env);
        storage::get_settlement(&env, settlement_id).ok_or(SettlementError::SettlementNotFound)
    }

    /// Read a paginated list of settlement records.
    pub fn paginate_settlements(env: Env, start_id: u64, limit: u32) -> Vec<SettlementRecord> {
        storage::extend_instance_ttl(&env);
        let total = storage::get_counter(&env);
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
