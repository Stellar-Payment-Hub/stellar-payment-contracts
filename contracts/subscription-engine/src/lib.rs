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
    pub fn status(_env: Env) -> Symbol {
        symbol_short!("sub_ok")
    }
}
