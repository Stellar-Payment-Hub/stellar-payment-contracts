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

    /// Query plan details by ID.
    pub fn get_plan(env: Env, plan_id: u64) -> Option<SubscriptionPlan> {
        storage::get_plan(&env, plan_id)
    }

    /// Subscribe an account to an active recurring billing plan.
    pub fn subscribe(
        env: Env,
        subscriber: Address,
        plan_id: u64,
    ) -> Result<u64, SubscriptionError> {
        subscriber.require_auth();
        let plan = storage::get_plan(&env, plan_id).ok_or(SubscriptionError::PlanNotFound)?;
        if !plan.active {
            return Err(SubscriptionError::PlanInactive);
        }
        if subscriber == plan.merchant {
            return Err(SubscriptionError::SameAddress);
        }

        let now = env.ledger().timestamp();
        let id = storage::increment_sub_count(&env);
        let sub = SubscriptionRecord {
            id,
            plan_id,
            subscriber: subscriber.clone(),
            merchant: plan.merchant.clone(),
            token: plan.token.clone(),
            status: SubscriptionStatus::Active,
            started_at: now,
            last_billed_at: 0,
            next_bill_at: now, // Due immediately for initial charge
            cycles_billed: 0,
            max_cycles: plan.max_cycles,
            total_paid: 0,
        };
        storage::set_subscription(&env, id, &sub);
        storage::adjust_merchant_stats(&env, &plan.merchant, 1, plan.amount);
        events::emit_subscription_started(&env, id, plan_id, &subscriber);
        storage::extend_sub_ttl(&env, id);
        Ok(id)
    }

    /// Query subscription record by ID.
    pub fn get_subscription(env: Env, id: u64) -> Option<SubscriptionRecord> {
        storage::get_subscription(&env, id)
    }

    /// Check if a subscription is currently due for recurring billing.
    pub fn is_charge_due(env: Env, sub_id: u64) -> bool {
        if let Some(sub) = storage::get_subscription(&env, sub_id) {
            if sub.status == SubscriptionStatus::Active {
                let now = env.ledger().timestamp();
                return now >= sub.next_bill_at && (sub.max_cycles == 0 || sub.cycles_billed < sub.max_cycles);
            }
        }
        false
    }

    /// Execute scheduled recurring charge on subscriber.
    pub fn charge_subscriber(
        env: Env,
        caller: Address,
        sub_id: u64,
    ) -> Result<u32, SubscriptionError> {
        caller.require_auth();
        let mut sub = storage::get_subscription(&env, sub_id).ok_or(SubscriptionError::SubscriptionNotFound)?;
        if sub.status == SubscriptionStatus::Paused {
            return Err(SubscriptionError::SubscriptionPaused);
        }
        if sub.status == SubscriptionStatus::Cancelled || sub.status == SubscriptionStatus::Expired {
            return Err(SubscriptionError::SubscriptionCancelled);
        }

        let now = env.ledger().timestamp();
        if now < sub.next_bill_at {
            return Err(SubscriptionError::BillingEpochNotDue);
        }
        if sub.max_cycles > 0 && sub.cycles_billed >= sub.max_cycles {
            sub.status = SubscriptionStatus::Expired;
            storage::set_subscription(&env, sub_id, &sub);
            return Err(SubscriptionError::MaxCyclesReached);
        }

        let plan = storage::get_plan(&env, sub.plan_id).ok_or(SubscriptionError::PlanNotFound)?;
        sub.cycles_billed += 1;
        sub.last_billed_at = now;
        sub.next_bill_at = now + plan.interval_seconds;
        sub.total_paid += plan.amount;

        if sub.max_cycles > 0 && sub.cycles_billed >= sub.max_cycles {
            sub.status = SubscriptionStatus::Expired;
        }

        storage::set_subscription(&env, sub_id, &sub);
        events::emit_subscription_billed(&env, sub_id, &sub.subscriber, plan.amount, sub.cycles_billed);
        storage::extend_sub_ttl(&env, sub_id);
        Ok(sub.cycles_billed)
    }

    pub fn status(_env: Env) -> Symbol {
        symbol_short!("sub_ok")
    }
}
