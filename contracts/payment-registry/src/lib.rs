#![no_std]
pub mod errors;
pub mod events;
pub mod storage;
pub mod types;

use errors::ContractError;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, String, Symbol};
use types::{Payment, PaymentStatus};

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
        Ok(())
    }

    /// Read contract status (Level 1 backwards compatibility).
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
        creator.require_auth();

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        if creator == recipient {
            return Err(ContractError::SameAddress);
        }

        let id = storage::increment_count(&env);
        let timestamp = env.ledger().timestamp();

        let payment = Payment {
            id,
            creator: creator.clone(),
            recipient: recipient.clone(),
            amount,
            memo,
            status: PaymentStatus::Pending,
            created_at: timestamp,
            updated_at: timestamp,
        };

        storage::set_payment(&env, id, &payment);
        events::emit_payment_created(&env, id, &creator, &recipient, amount);

        Ok(id)
    }

    /// Read a payment record by ID.
    pub fn get_payment(env: Env, id: u64) -> Result<Payment, ContractError> {
        storage::get_payment(&env, id).ok_or(ContractError::PaymentNotFound)
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
