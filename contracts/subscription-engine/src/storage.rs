use crate::types::{SubscriptionPlan, SubscriptionRecord};
use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
pub enum DataKey {
    Admin,
    Paused,
    PlanCounter,
    SubscriptionCounter,
    Plan(u64),
    Subscription(u64),
    MerchantSubscriberCount(Address),
    MerchantMRR(Address),
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&DataKey::Paused, &paused);
}

pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false)
}

pub fn increment_plan_count(env: &Env) -> u64 {
    let count: u64 = env
        .storage()
        .instance()
        .get(&DataKey::PlanCounter)
        .unwrap_or(0)
        + 1;
    env.storage().instance().set(&DataKey::PlanCounter, &count);
    count
}

pub fn increment_sub_count(env: &Env) -> u64 {
    let count: u64 = env
        .storage()
        .instance()
        .get(&DataKey::SubscriptionCounter)
        .unwrap_or(0)
        + 1;
    env.storage()
        .instance()
        .set(&DataKey::SubscriptionCounter, &count);
    count
}

pub fn set_plan(env: &Env, id: u64, plan: &SubscriptionPlan) {
    env.storage().persistent().set(&DataKey::Plan(id), plan);
}

pub fn get_plan(env: &Env, id: u64) -> Option<SubscriptionPlan> {
    env.storage().persistent().get(&DataKey::Plan(id))
}

pub fn set_subscription(env: &Env, id: u64, sub: &SubscriptionRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::Subscription(id), sub);
}

pub fn get_subscription(env: &Env, id: u64) -> Option<SubscriptionRecord> {
    env.storage().persistent().get(&DataKey::Subscription(id))
}

pub fn adjust_merchant_stats(env: &Env, merchant: &Address, count_delta: i32, mrr_delta: i128) {
    let cur_count: i32 = env
        .storage()
        .persistent()
        .get(&DataKey::MerchantSubscriberCount(merchant.clone()))
        .unwrap_or(0);
    let new_count = (cur_count + count_delta).max(0);
    env.storage().persistent().set(
        &DataKey::MerchantSubscriberCount(merchant.clone()),
        &new_count,
    );

    let cur_mrr: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::MerchantMRR(merchant.clone()))
        .unwrap_or(0);
    let new_mrr = (cur_mrr + mrr_delta).max(0);
    env.storage()
        .persistent()
        .set(&DataKey::MerchantMRR(merchant.clone()), &new_mrr);
}

pub fn get_merchant_subscriber_count(env: &Env, merchant: &Address) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::MerchantSubscriberCount(merchant.clone()))
        .unwrap_or(0) as u32
}

pub fn get_merchant_mrr(env: &Env, merchant: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::MerchantMRR(merchant.clone()))
        .unwrap_or(0)
}

pub fn extend_instance_ttl(env: &Env) {
    env.storage().instance().extend_ttl(50_000, 100_000);
}

pub fn extend_sub_ttl(env: &Env, id: u64) {
    env.storage()
        .persistent()
        .extend_ttl(&DataKey::Subscription(id), 50_000, 100_000);
}
