use soroban_sdk::{symbol_short, Address, Env};

pub fn emit_settlement_created(
    env: &Env,
    settlement_id: u64,
    payer: &Address,
    total_amount: i128,
    recipient_count: u32,
) {
    let topics = (symbol_short!("created"), settlement_id, payer.clone());
    env.events().publish(topics, (total_amount, recipient_count));
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
