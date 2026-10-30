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
    Timelock(u64),
    Arbitrator,
    MinSettlementThreshold,
    TotalVolumeSettled,
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

const INSTANCE_BUMP_THRESHOLD: u32 = 17280;
const INSTANCE_LIFETIME: u32 = 120960;

pub fn extend_instance_ttl(env: &Env) {
    env.storage().instance().extend_ttl(INSTANCE_BUMP_THRESHOLD, INSTANCE_LIFETIME);
}

pub fn extend_settlement_ttl(env: &Env, id: u64) {
    let key = DataKey::Settlement(id);
    env.storage().persistent().extend_ttl(&key, INSTANCE_BUMP_THRESHOLD, INSTANCE_LIFETIME);
    let rec_key = DataKey::Recipients(id);
    env.storage().persistent().extend_ttl(&rec_key, INSTANCE_BUMP_THRESHOLD, INSTANCE_LIFETIME);
}

pub fn set_milestone(env: &Env, id: u64, index: u32, milestone: &crate::types::SettlementMilestone) {
    let key = DataKey::MilestoneRecord(id, index);
    env.storage().persistent().set(&key, milestone);
}

pub fn get_milestone(env: &Env, id: u64, index: u32) -> Option<crate::types::SettlementMilestone> {
    let key = DataKey::MilestoneRecord(id, index);
    env.storage().persistent().get(&key)
}

pub fn is_reentrancy_locked(env: &Env) -> bool {
    env.storage().instance().get(&DataKey::ReentrancyLocked).unwrap_or(false)
}

pub fn set_reentrancy_lock(env: &Env, locked: bool) {
    env.storage().instance().set(&DataKey::ReentrancyLocked, &locked);
}


pub fn set_timelock(env: &Env, id: u64, lock: &crate::types::EscrowTimelock) {
    env.storage().persistent().set(&DataKey::Timelock(id), lock);
}

pub fn get_timelock(env: &Env, id: u64) -> Option<crate::types::EscrowTimelock> {
    env.storage().persistent().get(&DataKey::Timelock(id))
}

pub fn set_settlement_dispute(env: &Env, id: u64, dispute: &crate::types::SettlementDispute) {
    env.storage().persistent().set(&DataKey::DisputeRecord(id), dispute);
}

pub fn get_settlement_dispute(env: &Env, id: u64) -> Option<crate::types::SettlementDispute> {
    env.storage().persistent().get(&DataKey::DisputeRecord(id))
}

pub fn set_arbitrator(env: &Env, arbitrator: &Address) {
    env.storage().instance().set(&DataKey::Arbitrator, arbitrator);
}

pub fn get_arbitrator(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Arbitrator)
}

pub fn set_min_threshold(env: &Env, min: i128) {
    env.storage().instance().set(&DataKey::MinSettlementThreshold, &min);
}

pub fn get_min_threshold(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::MinSettlementThreshold).unwrap_or(1)
}

pub fn get_total_volume(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::TotalVolumeSettled).unwrap_or(0)
}

pub fn add_settled_volume(env: &Env, amount: i128) {
    let next = get_total_volume(env) + amount;
    env.storage().instance().set(&DataKey::TotalVolumeSettled, &next);
}
