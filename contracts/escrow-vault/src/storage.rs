use soroban_sdk::{contracttype, Address, Env};
use crate::types::{EscrowMilestone, EscrowRecord, VoteDecision};

#[contracttype]
pub enum DataKey {
    Admin,
    EscrowCounter,
    Escrow(u64),
    Vote(u64, Address),
    Milestone(u64, u32),
    MilestoneCount(u64),
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn increment_escrow_count(env: &Env) -> u64 {
    let count: u64 = env.storage().instance().get(&DataKey::EscrowCounter).unwrap_or(0) + 1;
    env.storage().instance().set(&DataKey::EscrowCounter, &count);
    count
}

pub fn set_escrow(env: &Env, id: u64, record: &EscrowRecord) {
    env.storage().persistent().set(&DataKey::Escrow(id), record);
}

pub fn get_escrow(env: &Env, id: u64) -> Option<EscrowRecord> {
    env.storage().persistent().get(&DataKey::Escrow(id))
}

pub fn set_vote(env: &Env, escrow_id: u64, voter: &Address, decision: VoteDecision) {
    env.storage().persistent().set(&DataKey::Vote(escrow_id, voter.clone()), &decision);
}

pub fn get_vote(env: &Env, escrow_id: u64, voter: &Address) -> VoteDecision {
    env.storage().persistent().get(&DataKey::Vote(escrow_id, voter.clone())).unwrap_or(VoteDecision::None)
}

pub fn set_milestone(env: &Env, escrow_id: u64, index: u32, milestone: &EscrowMilestone) {
    env.storage().persistent().set(&DataKey::Milestone(escrow_id, index), milestone);
}

pub fn get_milestone(env: &Env, escrow_id: u64, index: u32) -> Option<EscrowMilestone> {
    env.storage().persistent().get(&DataKey::Milestone(escrow_id, index))
}

pub fn extend_escrow_ttl(env: &Env, id: u64) {
    env.storage().persistent().extend_ttl(&DataKey::Escrow(id), 50_000, 100_000);
}
