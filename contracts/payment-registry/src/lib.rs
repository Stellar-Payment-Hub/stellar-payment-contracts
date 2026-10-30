#![no_std]
pub mod errors;
pub mod events;
pub mod storage;
pub mod types;

use errors::ContractError;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, String, Symbol};
use types::{Payment, PaymentAuditLog, PaymentCategory, PaymentStatus, Role};
use soroban_sdk::{BytesN, Vec};

#[contract]
pub struct PaymentRegistry;

#[contractimpl]
impl PaymentRegistry {
    /// Initialize the contract with an admin address.
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if storage::get_admin(&env).is_some() {
            return Err(ContractError::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        storage::set_role(&env, &admin, Role::Admin, true);
        storage::extend_instance_ttl(&env);
        Ok(())
    }

    /// Grant a protocol role to an account (Admin only).
    pub fn grant_role(env: Env, caller: Address, account: Address, role: Role) -> Result<(), ContractError> {
        caller.require_auth();
        let admin = storage::get_admin(&env).ok_or(ContractError::Unauthorized)?;
        if caller != admin && !storage::has_role(&env, &caller, Role::Admin) {
            return Err(ContractError::Unauthorized);
        }
        storage::set_role(&env, &account, role, true);
        events::emit_role_granted(&env, &account, role);
        storage::extend_instance_ttl(&env);
        Ok(())
    }

    /// Revoke a protocol role from an account (Admin only).
    pub fn revoke_role(env: Env, caller: Address, account: Address, role: Role) -> Result<(), ContractError> {
        caller.require_auth();
        let admin = storage::get_admin(&env).ok_or(ContractError::Unauthorized)?;
        if caller != admin && !storage::has_role(&env, &caller, Role::Admin) {
            return Err(ContractError::Unauthorized);
        }
        storage::set_role(&env, &account, role, false);
        events::emit_role_revoked(&env, &account, role);
        storage::extend_instance_ttl(&env);
        Ok(())
    }

    /// Check if an account possesses a specific protocol role.
    pub fn has_role(env: Env, account: Address, role: Role) -> bool {
        storage::has_role(&env, &account, role)
    }

    /// Pause the contract operations in an emergency (Admin or Operator).
    pub fn pause(env: Env, caller: Address) -> Result<(), ContractError> {
        caller.require_auth();
        let is_admin = storage::get_admin(&env).map(|a| a == caller).unwrap_or(false);
        let is_operator = storage::has_role(&env, &caller, Role::Operator);
        if !is_admin && !is_operator {
            return Err(ContractError::Unauthorized);
        }
        storage::set_paused(&env, true);
        events::emit_pause_toggled(&env, true, &caller);
        storage::extend_instance_ttl(&env);
        Ok(())
    }

    /// Unpause contract operations (Admin only).
    pub fn unpause(env: Env, caller: Address) -> Result<(), ContractError> {
        caller.require_auth();
        let is_admin = storage::get_admin(&env).map(|a| a == caller).unwrap_or(false);
        if !is_admin {
            return Err(ContractError::Unauthorized);
        }
        storage::set_paused(&env, false);
        events::emit_pause_toggled(&env, false, &caller);
        storage::extend_instance_ttl(&env);
        Ok(())
    }

    /// Check if the contract is currently paused.
    pub fn is_paused(env: Env) -> bool {
        storage::is_paused(&env)
    }

    /// Upgrade contract code to a new deployed WASM hash (Admin only).
    pub fn upgrade(env: Env, admin: Address, new_wasm_hash: BytesN<32>) -> Result<(), ContractError> {
        admin.require_auth();
        let current_admin = storage::get_admin(&env).ok_or(ContractError::Unauthorized)?;
        if admin != current_admin {
            return Err(ContractError::Unauthorized);
        }
        env.deployer().update_current_contract_wasm(new_wasm_hash.clone());
        events::emit_contract_upgraded(&env, &admin, &new_wasm_hash);
        Ok(())
    }

    /// Read contract status and availability.
    pub fn status(_env: Env) -> Symbol {
        symbol_short!("ready")
    }

    /// Read the total number of registered payments.
    pub fn get_payment_count(env: Env) -> u64 {
        storage::get_count(&env)
    }

    /// Create a new programmable payment record on-chain.
    pub fn create_payment(
        env: Env,
        creator: Address,
        recipient: Address,
        amount: i128,
        memo: String,
    ) -> Result<u64, ContractError> {
        if storage::is_paused(&env) {
            return Err(ContractError::ContractPaused);
        }
        creator.require_auth();

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        if creator == recipient {
            return Err(ContractError::SameAddress);
        }

        storage::extend_instance_ttl(&env);
        let id = storage::increment_count(&env);
        let timestamp = env.ledger().timestamp();

        let payment = Payment {
            id,
            creator: creator.clone(),
            recipient: recipient.clone(),
            amount,
            token: None,
            category: PaymentCategory::Standard,
            memo,
            memo_hash: None,
            status: PaymentStatus::Pending,
            deadline: None,
            created_at: timestamp,
            updated_at: timestamp,
        };

        storage::set_payment(&env, id, &payment);
        storage::extend_payment_ttl(&env, id);
        events::emit_payment_created(&env, id, &creator, &recipient, amount);

        Ok(id)
    }

    /// Batch register multiple payment entries in an atomic call for gas optimization.
    pub fn create_payments_batch(
        env: Env,
        creator: Address,
        recipients: Vec<Address>,
        amounts: Vec<i128>,
        memo: String,
    ) -> Result<Vec<u64>, ContractError> {
        if storage::is_paused(&env) {
            return Err(ContractError::ContractPaused);
        }
        creator.require_auth();

        let len = recipients.len();
        if len == 0 {
            return Err(ContractError::BatchEmpty);
        }
        if len > 50 {
            return Err(ContractError::BatchTooLarge);
        }
        if len != amounts.len() {
            return Err(ContractError::InvalidAmount);
        }

        storage::extend_instance_ttl(&env);
        let timestamp = env.ledger().timestamp();
        let mut created_ids = Vec::new(&env);

        for i in 0..len {
            let recipient = recipients.get(i).unwrap();
            let amount = amounts.get(i).unwrap();

            if amount <= 0 {
                return Err(ContractError::InvalidAmount);
            }
            if creator == recipient {
                return Err(ContractError::SameAddress);
            }

            let id = storage::increment_count(&env);
            let payment = Payment {
                id,
                creator: creator.clone(),
                recipient: recipient.clone(),
                amount,
                token: None,
                category: PaymentCategory::Disbursement,
                memo: memo.clone(),
                memo_hash: None,
                status: PaymentStatus::Pending,
                deadline: None,
                created_at: timestamp,
                updated_at: timestamp,
            };

            storage::set_payment(&env, id, &payment);
            storage::extend_payment_ttl(&env, id);
            events::emit_payment_created(&env, id, &creator, &recipient, amount);
            created_ids.push_back(id);
        }

        events::emit_batch_payments_created(&env, &creator, len);
        Ok(created_ids)
    }

    /// Create a payment with cryptographic invoice memo hash verification.
    pub fn create_payment_with_hash(
        env: Env,
        creator: Address,
        recipient: Address,
        amount: i128,
        memo: String,
        memo_hash: BytesN<32>,
        category: PaymentCategory,
    ) -> Result<u64, ContractError> {
        if storage::is_paused(&env) {
            return Err(ContractError::ContractPaused);
        }
        creator.require_auth();

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }
        if creator == recipient {
            return Err(ContractError::SameAddress);
        }

        storage::extend_instance_ttl(&env);
        let id = storage::increment_count(&env);
        let timestamp = env.ledger().timestamp();

        let payment = Payment {
            id,
            creator: creator.clone(),
            recipient: recipient.clone(),
            amount,
            token: None,
            category,
            memo,
            memo_hash: Some(memo_hash.clone()),
            status: PaymentStatus::Pending,
            deadline: None,
            created_at: timestamp,
            updated_at: timestamp,
        };

        storage::set_payment(&env, id, &payment);
        storage::extend_payment_ttl(&env, id);
        events::emit_payment_created(&env, id, &creator, &recipient, amount);
        events::emit_memo_hash_bound(&env, id, &memo_hash);

        Ok(id)
    }

    /// Create an expirable payment record that automatically invalidates after a timestamp.
    pub fn create_expirable_payment(
        env: Env,
        creator: Address,
        recipient: Address,
        amount: i128,
        memo: String,
        deadline: u64,
    ) -> Result<u64, ContractError> {
        if storage::is_paused(&env) {
            return Err(ContractError::ContractPaused);
        }
        creator.require_auth();

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }
        if creator == recipient {
            return Err(ContractError::SameAddress);
        }

        let now = env.ledger().timestamp();
        if deadline <= now {
            return Err(ContractError::InvalidDeadline);
        }

        storage::extend_instance_ttl(&env);
        let id = storage::increment_count(&env);

        let payment = Payment {
            id,
            creator: creator.clone(),
            recipient: recipient.clone(),
            amount,
            token: None,
            category: PaymentCategory::Invoice,
            memo,
            memo_hash: None,
            status: PaymentStatus::Pending,
            deadline: Some(deadline),
            created_at: now,
            updated_at: now,
        };

        storage::set_payment(&env, id, &payment);
        storage::extend_payment_ttl(&env, id);
        events::emit_payment_created(&env, id, &creator, &recipient, amount);

        Ok(id)
    }

    /// Check and apply expiration status if current ledger timestamp exceeds deadline.
    pub fn check_and_expire(env: Env, id: u64) -> Result<bool, ContractError> {
        let mut payment = storage::get_payment(&env, id).ok_or(ContractError::PaymentNotFound)?;
        let now = env.ledger().timestamp();

        if let Some(deadline) = payment.deadline {
            if now >= deadline && payment.status == PaymentStatus::Pending {
                payment.status = PaymentStatus::Expired;
                payment.updated_at = now;
                storage::set_payment(&env, id, &payment);
                events::emit_payment_expired(&env, id, now);
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Read a payment record by ID.
    pub fn get_payment(env: Env, id: u64) -> Result<Payment, ContractError> {
        storage::extend_instance_ttl(&env);
        storage::get_payment(&env, id).ok_or(ContractError::PaymentNotFound)
    }

    /// Query a paginated range of payments (start_id inclusive, max_count items).
    pub fn paginate_payments(env: Env, start_id: u64, limit: u32) -> Vec<Payment> {
        storage::extend_instance_ttl(&env);
        let total = storage::get_count(&env);
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
            if let Some(payment) = storage::get_payment(&env, id) {
                results.push_back(payment);
            }
        }

        results
    }

    /// Update payment status with strict state machine validation.
    pub fn update_payment_status(
        env: Env,
        caller: Address,
        id: u64,
        new_status: PaymentStatus,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let mut payment = storage::get_payment(&env, id).ok_or(ContractError::PaymentNotFound)?;

        // Only creator, recipient, or admin can update status
        let is_creator = caller == payment.creator;
        let is_recipient = caller == payment.recipient;
        let is_admin = storage::get_admin(&env).map(|a| a == caller).unwrap_or(false);

        if !is_creator && !is_recipient && !is_admin {
            return Err(ContractError::Unauthorized);
        }

        // Validate state transitions
        match payment.status {
            PaymentStatus::Completed => return Err(ContractError::AlreadyCompleted),
            PaymentStatus::Cancelled => return Err(ContractError::AlreadyCancelled),
            PaymentStatus::Failed | PaymentStatus::Expired => {
                return Err(ContractError::InvalidTransition)
            }
            PaymentStatus::Pending => {
                // Pending can transition to Processing, Completed, Cancelled, Failed, Expired
            }
            PaymentStatus::Processing => {
                // Processing can transition to Completed, Cancelled, Failed
                if new_status == PaymentStatus::Pending {
                    return Err(ContractError::InvalidTransition);
                }
            }
        }

        let timestamp = env.ledger().timestamp();
        payment.status = new_status;
        payment.updated_at = timestamp;

        storage::set_payment(&env, id, &payment);
        events::emit_payment_updated(&env, id, new_status, timestamp);

        if new_status == PaymentStatus::Completed {
            events::emit_payment_completed(&env, id, timestamp);
        } else if new_status == PaymentStatus::Cancelled {
            events::emit_payment_cancelled(&env, id, timestamp);
        }

        Ok(())
    }

    /// Mark payment as completed.
    pub fn complete_payment(env: Env, caller: Address, id: u64) -> Result<(), ContractError> {
        Self::update_payment_status(env, caller, id, PaymentStatus::Completed)
    }

    /// Cancel a payment before completion.
    pub fn cancel_payment(env: Env, caller: Address, id: u64) -> Result<(), ContractError> {
        Self::update_payment_status(env, caller, id, PaymentStatus::Cancelled)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, String};

    #[test]
    fn test_create_and_read_payment() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(PaymentRegistry, ());
        let client = PaymentRegistryClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let recipient = Address::generate(&env);
        let memo = String::from_str(&env, "Invoice #1001");

        let id = client.create_payment(&creator, &recipient, &100_000_000, &memo);
        assert_eq!(id, 1);
        assert_eq!(client.get_payment_count(), 1);

        let payment = client.get_payment(&1);
        assert_eq!(payment.id, 1);
        assert_eq!(payment.amount, 100_000_000);
        assert_eq!(payment.status, PaymentStatus::Pending);
        assert_eq!(payment.creator, creator);
        assert_eq!(payment.recipient, recipient);
    }

    #[test]
    fn test_update_and_complete_payment() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(PaymentRegistry, ());
        let client = PaymentRegistryClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let recipient = Address::generate(&env);
        let memo = String::from_str(&env, "Lunch bill");

        let id = client.create_payment(&creator, &recipient, &50_000_000, &memo);

        // Transition: Pending -> Processing
        client.update_payment_status(&creator, &id, &PaymentStatus::Processing);
        let p1 = client.get_payment(&id);
        assert_eq!(p1.status, PaymentStatus::Processing);

        // Transition: Processing -> Completed
        client.complete_payment(&recipient, &id);
        let p2 = client.get_payment(&id);
        assert_eq!(p2.status, PaymentStatus::Completed);
    }

    #[test]
    fn test_cancel_payment() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(PaymentRegistry, ());
        let client = PaymentRegistryClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let recipient = Address::generate(&env);
        let memo = String::from_str(&env, "Cancel test");

        let id = client.create_payment(&creator, &recipient, &25_000_000, &memo);
        client.cancel_payment(&creator, &id);

        let payment = client.get_payment(&id);
        assert_eq!(payment.status, PaymentStatus::Cancelled);
    }

    #[test]
    fn test_invalid_amount_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(PaymentRegistry, ());
        let client = PaymentRegistryClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let recipient = Address::generate(&env);
        let memo = String::from_str(&env, "Zero amount");

        let result = client.try_create_payment(&creator, &recipient, &0, &memo);
        assert_eq!(result, Err(Ok(ContractError::InvalidAmount)));
    }

    #[test]
    fn test_same_address_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(PaymentRegistry, ());
        let client = PaymentRegistryClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let memo = String::from_str(&env, "Self pay");

        let result = client.try_create_payment(&creator, &creator, &100_000_000, &memo);
        assert_eq!(result, Err(Ok(ContractError::SameAddress)));
    }
}
