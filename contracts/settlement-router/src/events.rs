use soroban_sdk::{symbol_short, Address, Env};

pub fn emit_settlement_created(
    env: &Env,
    settlement_id: u64,
    payer: &Address,
    total_amount: i128,
    recipient_count: u32,
) {
    let topics = (soroban_sdk::symbol_short!("settle"), soroban_sdk::symbol_short!("created"), settlement_id);
    env.events().publish(topics, (total_amount, recipient_count, payer.clone()));
}

pub fn emit_recipient_processed(
    env: &Env,
    settlement_id: u64,
    recipient: &Address,
    amount: i128,
    sub_payment_id: u64,
) {
    let topics = (symbol_short!("sub_pay"), settlement_id, recipient.clone());
    env.events().publish(topics, (amount, sub_payment_id));
}

pub fn emit_settlement_completed(
    env: &Env,
    settlement_id: u64,
    payer: &Address,
    total_amount: i128,
) {
    let topics = (symbol_short!("completed"), settlement_id, payer.clone());
    env.events().publish(topics, total_amount);
}

pub fn emit_settlement_cancelled(
    env: &Env,
    settlement_id: u64,
    payer: &Address,
) {
    let topics = (symbol_short!("cancelled"), settlement_id, payer.clone());
    env.events().publish(topics, symbol_short!("cancelled"));
}

pub fn emit_settlement_refunded(
    env: &Env,
    settlement_id: u64,
    payer: &Address,
    amount: i128,
) {
    let topics = (soroban_sdk::symbol_short!("refund"), settlement_id, payer.clone());
    env.events().publish(topics, amount);
}

pub fn emit_router_upgraded(env: &Env, admin: &Address, new_wasm_hash: &soroban_sdk::BytesN<32>) {
    let topics = (soroban_sdk::symbol_short!("router"), soroban_sdk::symbol_short!("upgraded"));
    env.events().publish(topics, (admin.clone(), new_wasm_hash.clone()));
}

pub fn emit_router_paused(env: &Env, admin: &Address, paused: bool) {
    let topics = (soroban_sdk::symbol_short!("circuit"), soroban_sdk::symbol_short!("pause"));
    env.events().publish(topics, (admin.clone(), paused));
}

pub fn emit_dispute_raised(env: &Env, settlement_id: u64, initiator: &Address, evidence_hash: &soroban_sdk::BytesN<32>) {
    let topics = (soroban_sdk::symbol_short!("dispute"), settlement_id, initiator.clone());
    env.events().publish(topics, evidence_hash.clone());
}

pub fn emit_dispute_resolved(env: &Env, settlement_id: u64, verdict: u32) {
    let topics = (soroban_sdk::symbol_short!("verdict"), settlement_id);
    env.events().publish(topics, verdict);
}

pub fn emit_milestone_released(env: &Env, settlement_id: u64, index: u32, amount: i128) {
    let topics = (soroban_sdk::symbol_short!("milestone"), settlement_id);
    env.events().publish(topics, (index, amount));
}


pub fn emit_token_whitelist_updated(env: &Env, token: &Address, enabled: bool, min_deposit: i128) {
    let topics = (soroban_sdk::symbol_short!("token_wl"), token.clone());
    env.events().publish(topics, (enabled, min_deposit));
}

pub fn emit_protocol_fee_collected(env: &Env, settlement_id: u64, token: &Address, fee_amount: i128) {
    let topics = (soroban_sdk::symbol_short!("fee_col"), settlement_id, token.clone());
    env.events().publish(topics, fee_amount);
}

pub fn emit_fee_withdrawn(env: &Env, token: &Address, recipient: &Address, amount: i128) {
    let topics = (soroban_sdk::symbol_short!("fee_wd"), token.clone(), recipient.clone());
    env.events().publish(topics, amount);
}
