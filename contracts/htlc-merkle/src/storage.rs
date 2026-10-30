use crate::types::{HtlcRecord, MerkleDistribution};
use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
pub enum DataKey {
    Admin,
    HtlcCounter,
    DistCounter,
    Htlc(u64),
    Distribution(u64),
    ClaimedLeaf(u64, u32),
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn increment_htlc_count(env: &Env) -> u64 {
    let count: u64 = env
        .storage()
        .instance()
        .get(&DataKey::HtlcCounter)
        .unwrap_or(0)
        + 1;
    env.storage().instance().set(&DataKey::HtlcCounter, &count);
    count
}

pub fn increment_dist_count(env: &Env) -> u64 {
    let count: u64 = env
        .storage()
        .instance()
        .get(&DataKey::DistCounter)
        .unwrap_or(0)
        + 1;
    env.storage().instance().set(&DataKey::DistCounter, &count);
    count
}

pub fn set_htlc(env: &Env, id: u64, record: &HtlcRecord) {
    env.storage().persistent().set(&DataKey::Htlc(id), record);
}

pub fn get_htlc(env: &Env, id: u64) -> Option<HtlcRecord> {
    env.storage().persistent().get(&DataKey::Htlc(id))
}

pub fn set_dist(env: &Env, id: u64, dist: &MerkleDistribution) {
    env.storage()
        .persistent()
        .set(&DataKey::Distribution(id), dist);
}

pub fn get_dist(env: &Env, id: u64) -> Option<MerkleDistribution> {
    env.storage().persistent().get(&DataKey::Distribution(id))
}

pub fn is_leaf_claimed(env: &Env, dist_id: u64, index: u32) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::ClaimedLeaf(dist_id, index))
        .unwrap_or(false)
}

pub fn set_leaf_claimed(env: &Env, dist_id: u64, index: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::ClaimedLeaf(dist_id, index), &true);
}

pub fn extend_htlc_ttl(env: &Env, id: u64) {
    env.storage()
        .persistent()
        .extend_ttl(&DataKey::Htlc(id), 50_000, 100_000);
}
