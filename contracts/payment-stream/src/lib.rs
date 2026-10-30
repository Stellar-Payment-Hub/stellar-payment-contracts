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

    /// Query full stream details by ID.
    pub fn get_stream(env: Env, id: u64) -> Option<StreamRecord> {
        storage::get_stream(&env, id)
    }

    /// Compute current accrued tokens available for withdrawal.
    pub fn calculate_withdrawable(env: Env, id: u64) -> i128 {
        let stream = match storage::get_stream(&env, id) {
            Some(s) => s,
            None => return 0,
        };
        if stream.status == StreamStatus::Cancelled || stream.status == StreamStatus::Completed {
            return 0;
        }

        let now = env.ledger().timestamp();
        if stream.cliff_time > 0 && now < stream.cliff_time {
            return 0; // Cliff not yet reached
        }

        let effective_now = if stream.status == StreamStatus::Paused {
            stream.paused_at
        } else {
            now
        };

        let effective_stop = stream.stop_time + stream.pause_duration;
        if effective_now >= effective_stop {
            return stream.deposit_amount - stream.withdrawn_amount;
        }

        let elapsed = effective_now - stream.start_time - stream.pause_duration;
        let total_accrued = (elapsed as i128) * stream.rate_per_second;
        let available = total_accrued - stream.withdrawn_amount;
        available.max(0).min(stream.deposit_amount - stream.withdrawn_amount)
    }

    /// Query whether cliff period has been unlocked.
    pub fn is_cliff_passed(env: Env, id: u64) -> bool {
        if let Some(s) = storage::get_stream(&env, id) {
            return s.cliff_time == 0 || env.ledger().timestamp() >= s.cliff_time;
        }
        false
    }

    /// Claim accrued withdrawable tokens (Recipient only).
    pub fn withdraw_from_stream(env: Env, recipient: Address, id: u64) -> Result<i128, StreamError> {
        recipient.require_auth();
        let mut stream = storage::get_stream(&env, id).ok_or(StreamError::StreamNotFound)?;
        if stream.recipient != recipient {
            return Err(StreamError::Unauthorized);
        }
        if stream.status != StreamStatus::Active && stream.status != StreamStatus::Paused {
            return Err(StreamError::StreamNotActive);
        }

        let withdrawable = Self::calculate_withdrawable(env.clone(), id);
        if withdrawable <= 0 {
            return Err(StreamError::ZeroWithdrawable);
        }

        stream.withdrawn_amount += withdrawable;
        if stream.withdrawn_amount >= stream.deposit_amount {
            stream.status = StreamStatus::Completed;
        }
        storage::set_stream(&env, id, &stream);
        events::emit_stream_withdrawn(&env, id, &recipient, withdrawable);
        if stream.status == StreamStatus::Completed {
            events::emit_stream_status(&env, id, StreamStatus::Completed);
        }
        storage::extend_stream_ttl(&env, id);
        Ok(withdrawable)
    }

    /// Pause an active stream (Sender only).
    pub fn pause_stream(env: Env, sender: Address, id: u64) -> Result<(), StreamError> {
        sender.require_auth();
        let mut stream = storage::get_stream(&env, id).ok_or(StreamError::StreamNotFound)?;
        if stream.sender != sender {
            return Err(StreamError::Unauthorized);
        }
        if stream.status != StreamStatus::Active {
            return Err(StreamError::StreamNotActive);
        }
        stream.status = StreamStatus::Paused;
        stream.paused_at = env.ledger().timestamp();
        storage::set_stream(&env, id, &stream);
        events::emit_stream_status(&env, id, StreamStatus::Paused);
        storage::extend_stream_ttl(&env, id);
        Ok(())
    }

    pub fn status(_env: Env) -> Symbol {
        symbol_short!("stream_ok")
    }
}
