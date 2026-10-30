pub mod errors;
pub mod events;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

use errors::StreamError;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol, Vec};
use types::{BatchStreamItem, StreamRecord, StreamStatus};

#[contract]
pub struct PaymentStream;

#[contractimpl]
impl PaymentStream {
    /// Initialize contract with admin authority.
    pub fn initialize(env: Env, admin: Address) -> Result<(), StreamError> {
        if storage::get_admin(&env).is_some() {
            return Err(StreamError::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        env.storage().instance().extend_ttl(50_000, 100_000);
        Ok(())
    }

    /// Health check status.
    pub fn status(_env: Env) -> Symbol {
        symbol_short!("stream_ok")
    }
}
