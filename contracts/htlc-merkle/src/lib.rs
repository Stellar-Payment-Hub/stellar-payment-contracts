pub mod errors;
pub mod events;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

use errors::HtlcError;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Bytes, BytesN, Env, Symbol, Vec};
use types::{HtlcRecord, HtlcStatus, MerkleDistribution};

#[contract]
pub struct HtlcMerkleVault;

#[contractimpl]
impl HtlcMerkleVault {
    /// Initialize contract with admin authority.
    pub fn initialize(env: Env, admin: Address) -> Result<(), HtlcError> {
        if storage::get_admin(&env).is_some() {
            return Err(HtlcError::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        env.storage().instance().extend_ttl(50_000, 100_000);
        Ok(())
    }

    /// Health check status.
    pub fn status(_env: Env) -> Symbol {
        symbol_short!("htlc_ok")
    }
}
