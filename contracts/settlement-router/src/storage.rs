use soroban_sdk::{contracttype, Address, Env, Vec};
use crate::types::{RecipientShare, SettlementRecord};

#[contracttype]
pub enum DataKey {
    Admin,
    RegistryContract,
    TreasuryAccount,
    FeeConfiguration,
    SettlementCounter,
    Settlement(u64),
    Recipients(u64),
    Paused,
    ReentrancyLocked,
    DisputeRecord(u64),
    MilestoneRecord(u64, u32),
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


pub fn set_treasury(env: &Env, treasury: &Address) {
    env.storage().instance().set(&DataKey::TreasuryAccount, treasury);
}

pub fn get_treasury(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::TreasuryAccount)
}

pub fn set_fee_config(env: &Env, config: &crate::types::FeeConfig) {
    env.storage().instance().set(&DataKey::FeeConfiguration, config);
}

pub fn get_fee_config(env: &Env) -> Option<crate::types::FeeConfig> {
    env.storage().instance().get(&DataKey::FeeConfiguration)
}

pub fn is_paused(env: &Env) -> bool {
    env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
}

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&DataKey::Paused, &paused);
}
