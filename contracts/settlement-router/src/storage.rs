use soroban_sdk::{contracttype, Address, Env, Vec};
use crate::types::{RecipientShare, SettlementRecord};

#[contracttype]
pub enum DataKey {
    Admin,
    RegistryContract,
    SettlementCounter,
    Settlement(u64),
    Recipients(u64),
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn set_registry(env: &Env, registry: &Address) {
    env.storage().instance().set(&DataKey::RegistryContract, registry);
}

pub fn get_registry(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::RegistryContract)
}

pub fn get_counter(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::SettlementCounter).unwrap_or(0)
}

pub fn increment_counter(env: &Env) -> u64 {
    let next = get_counter(env) + 1;
    env.storage().instance().set(&DataKey::SettlementCounter, &next);
    next
}

pub fn set_settlement(env: &Env, id: u64, record: &SettlementRecord) {
    env.storage().persistent().set(&DataKey::Settlement(id), record);
}

pub fn get_settlement(env: &Env, id: u64) -> Option<SettlementRecord> {
    env.storage().persistent().get(&DataKey::Settlement(id))
}

pub fn set_recipients(env: &Env, id: u64, recipients: &Vec<RecipientShare>) {
    env.storage().persistent().set(&DataKey::Recipients(id), recipients);
}

pub fn get_recipients(env: &Env, id: u64) -> Option<Vec<RecipientShare>> {
    env.storage().persistent().get(&DataKey::Recipients(id))
}
