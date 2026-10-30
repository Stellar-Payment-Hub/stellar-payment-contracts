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
    pub fn status(_env: Env) -> Symbol {
        symbol_short!("escrow_ok")
    }
}
