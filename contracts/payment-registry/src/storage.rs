use soroban_sdk::{contracttype, Address, Env};
use crate::types::Payment;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Counter,
    Payment(u64),
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_count(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::Counter).unwrap_or(0)
}

pub fn increment_count(env: &Env) -> u64 {
    let next = get_count(env) + 1;
    env.storage().instance().set(&DataKey::Counter, &next);
    next
}

pub fn get_payment(env: &Env, id: u64) -> Option<Payment> {
    env.storage().persistent().get(&DataKey::Payment(id))
}

pub fn set_payment(env: &Env, id: u64, payment: &Payment) {
    env.storage().persistent().set(&DataKey::Payment(id), payment);
}
