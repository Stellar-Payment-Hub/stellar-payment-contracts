use soroban_sdk::{contracttype, Address, Env};
use crate::types::StreamRecord;

#[contracttype]
pub enum DataKey {
    Admin,
    Paused,
    StreamCounter,
    Stream(u64),
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
    env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
}

pub fn increment_stream_count(env: &Env) -> u64 {
    let count: u64 = env.storage().instance().get(&DataKey::StreamCounter).unwrap_or(0) + 1;
    env.storage().instance().set(&DataKey::StreamCounter, &count);
    count
}

pub fn set_stream(env: &Env, id: u64, record: &StreamRecord) {
    env.storage().persistent().set(&DataKey::Stream(id), record);
}

pub fn get_stream(env: &Env, id: u64) -> Option<StreamRecord> {
    env.storage().persistent().get(&DataKey::Stream(id))
}

pub fn extend_stream_ttl(env: &Env, id: u64) {
    env.storage().persistent().extend_ttl(&DataKey::Stream(id), 50_000, 100_000);
}
