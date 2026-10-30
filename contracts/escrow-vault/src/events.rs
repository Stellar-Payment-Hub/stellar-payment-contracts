use soroban_sdk::{symbol_short, Address, BytesN, Env};
use crate::types::EscrowStatus;

pub fn emit_escrow_created(env: &Env, id: u64, buyer: &Address, seller: &Address, amount: i128) {
    let topics = (symbol_short!("escrow"), symbol_short!("created"), id);
    env.events().publish(topics, (buyer.clone(), seller.clone(), amount));
}

pub fn emit_vote_cast(env: &Env, id: u64, voter: &Address, decision: u32) {
    let topics = (symbol_short!("vote"), id, voter.clone());
    env.events().publish(topics, decision);
}

pub fn emit_escrow_status(env: &Env, id: u64, status: EscrowStatus) {
    let topics = (symbol_short!("esc_stat"), id);
    env.events().publish(topics, status as u32);
}

pub fn emit_evidence_submitted(env: &Env, id: u64, index: u32, evidence_hash: &BytesN<32>) {
    let topics = (symbol_short!("evidence"), id, index);
    env.events().publish(topics, evidence_hash.clone());
}
