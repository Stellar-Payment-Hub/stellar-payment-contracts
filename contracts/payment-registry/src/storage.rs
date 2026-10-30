use soroban_sdk::{contracttype, Address, Env};
use crate::types::Payment;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Counter,
    Payment(u64),
    Role(Address, u32),
    Paused,
    ReentrancyLocked,
    Nonce(Address),
    PaymentAudit(u64),
    PaymentDispute(u64),
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


pub fn has_role(env: &Env, account: &Address, role: crate::types::Role) -> bool {
    let key = DataKey::Role(account.clone(), role as u32);
    env.storage().instance().get(&key).unwrap_or(false)
}

pub fn set_role(env: &Env, account: &Address, role: crate::types::Role, enabled: bool) {
    let key = DataKey::Role(account.clone(), role as u32);
    if enabled {
        env.storage().instance().set(&key, &true);
    } else {
        env.storage().instance().remove(&key);
    }
}

pub fn is_paused(env: &Env) -> bool {
    env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
}

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&DataKey::Paused, &paused);
}

const INSTANCE_BUMP_THRESHOLD: u32 = 17280; // ~1 day in ledgers (5s/ledger)
const INSTANCE_LIFETIME: u32 = 120960;      // ~7 days in ledgers

pub fn extend_instance_ttl(env: &Env) {
    env.storage().instance().extend_ttl(INSTANCE_BUMP_THRESHOLD, INSTANCE_LIFETIME);
}

pub fn extend_payment_ttl(env: &Env, id: u64) {
    let key = DataKey::Payment(id);
    env.storage().persistent().extend_ttl(&key, INSTANCE_BUMP_THRESHOLD, INSTANCE_LIFETIME);
}

pub fn set_audit_log(env: &Env, id: u64, log: &crate::types::PaymentAuditLog) {
    env.storage().persistent().set(&DataKey::PaymentAudit(id), log);
}

pub fn get_audit_log(env: &Env, id: u64) -> Option<crate::types::PaymentAuditLog> {
    env.storage().persistent().get(&DataKey::PaymentAudit(id))
}

pub fn is_reentrancy_locked(env: &Env) -> bool {
    env.storage().instance().get(&DataKey::ReentrancyLocked).unwrap_or(false)
}

pub fn set_reentrancy_lock(env: &Env, locked: bool) {
    env.storage().instance().set(&DataKey::ReentrancyLocked, &locked);
}

pub fn get_nonce(env: &Env, account: &Address) -> u64 {
    env.storage().instance().get(&DataKey::Nonce(account.clone())).unwrap_or(0)
}

pub fn increment_nonce(env: &Env, account: &Address) -> u64 {
    let next = get_nonce(env, account) + 1;
    env.storage().instance().set(&DataKey::Nonce(account.clone()), &next);
    next
}

pub fn get_dispute(env: &Env, payment_id: u64) -> Option<crate::types::PaymentDispute> {
    env.storage().persistent().get(&DataKey::PaymentDispute(payment_id))
}

pub fn set_dispute(env: &Env, payment_id: u64, dispute: &crate::types::PaymentDispute) {
    env.storage().persistent().set(&DataKey::PaymentDispute(payment_id), dispute);
}
