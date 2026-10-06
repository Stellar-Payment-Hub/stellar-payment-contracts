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
