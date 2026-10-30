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
    /// Open a continuous token stream with by-the-second accrual.
    pub fn create_stream(
        env: Env,
        sender: Address,
        recipient: Address,
        token: Address,
        deposit_amount: i128,
        duration_seconds: u64,
        cliff_duration_seconds: u64,
    ) -> Result<u64, StreamError> {
        sender.require_auth();
        if deposit_amount <= 0 {
            return Err(StreamError::InvalidAmount);
        }
        if sender == recipient {
            return Err(StreamError::SamePartyAddress);
        }
        if duration_seconds < 60 { // Minimum 1 minute
            return Err(StreamError::InvalidDuration);
        }
        if cliff_duration_seconds > duration_seconds {
            return Err(StreamError::CliffPastStopTime);
        }

        let now = env.ledger().timestamp();
        let stop_time = now + duration_seconds;
        let cliff_time = if cliff_duration_seconds > 0 { now + cliff_duration_seconds } else { 0 };
        let rate_per_second = deposit_amount / (duration_seconds as i128);

        let id = storage::increment_stream_count(&env);
        let record = StreamRecord {
            id,
            sender: sender.clone(),
            recipient: recipient.clone(),
            token,
            deposit_amount,
            withdrawn_amount: 0,
            start_time: now,
            stop_time,
            cliff_time,
            rate_per_second,
            status: StreamStatus::Active,
            paused_at: 0,
            pause_duration: 0,
        };
        storage::set_stream(&env, id, &record);
        events::emit_stream_created(&env, id, &sender, &recipient, deposit_amount);
        storage::extend_stream_ttl(&env, id);
        Ok(id)
    }

    pub fn status(_env: Env) -> Symbol {
        symbol_short!("stream_ok")
    }
}
