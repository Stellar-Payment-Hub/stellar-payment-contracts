pub mod errors;
pub mod events;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

use errors::SubscriptionError;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, String, Symbol};
use types::{SubscriptionPlan, SubscriptionRecord, SubscriptionStatus};

#[contract]
pub struct SubscriptionEngine;

#[contractimpl]
impl SubscriptionEngine {
    /// Initialize contract with admin authority.
    pub fn initialize(env: Env, admin: Address) -> Result<(), SubscriptionError> {
        if storage::get_admin(&env).is_some() {
            return Err(SubscriptionError::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        storage::extend_instance_ttl(&env);
        Ok(())
    }

    /// Health check status.
    /// Create a recurring billing plan for a merchant.
    pub fn create_plan(
        env: Env,
        merchant: Address,
        token: Address,
        name: String,
        amount: i128,
        interval_seconds: u64,
        max_cycles: u32,
    ) -> Result<u64, SubscriptionError> {
        merchant.require_auth();
        if amount <= 0 {
            return Err(SubscriptionError::InvalidAmount);
        }
        if interval_seconds < 60 { // Minimum 1 minute interval
            return Err(SubscriptionError::InvalidInterval);
        }

        let id = storage::increment_plan_count(&env);
        let now = env.ledger().timestamp();
        let plan = SubscriptionPlan {
            id,
            merchant: merchant.clone(),
            token,
            name,
            amount,
            interval_seconds,
            max_cycles,
            active: true,
            created_at: now,
        };
        storage::set_plan(&env, id, &plan);
        events::emit_plan_created(&env, id, &merchant, amount, interval_seconds);
        storage::extend_instance_ttl(&env);
        Ok(id)
    }

    pub fn status(_env: Env) -> Symbol {
        symbol_short!("sub_ok")
    }
}
