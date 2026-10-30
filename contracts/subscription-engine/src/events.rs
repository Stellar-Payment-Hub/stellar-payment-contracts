use crate::types::SubscriptionStatus;
use soroban_sdk::{symbol_short, Address, Env};

pub fn emit_plan_created(env: &Env, plan_id: u64, merchant: &Address, amount: i128, interval: u64) {
    let topics = (symbol_short!("plan_new"), plan_id, merchant.clone());
    env.events().publish(topics, (amount, interval));
}

pub fn emit_subscription_started(env: &Env, sub_id: u64, plan_id: u64, subscriber: &Address) {
    let topics = (symbol_short!("sub_start"), sub_id, subscriber.clone());
    env.events().publish(topics, plan_id);
}

pub fn emit_subscription_billed(
    env: &Env,
    sub_id: u64,
    subscriber: &Address,
    amount: i128,
    cycle: u32,
) {
    let topics = (symbol_short!("sub_bill"), sub_id, subscriber.clone());
    env.events().publish(topics, (amount, cycle));
}

pub fn emit_subscription_status(env: &Env, sub_id: u64, status: SubscriptionStatus) {
    let topics = (symbol_short!("sub_stat"), sub_id);
    env.events().publish(topics, status as u32);
}
