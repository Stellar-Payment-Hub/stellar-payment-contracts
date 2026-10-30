use soroban_sdk::{symbol_short, Address, Env};
use crate::types::PaymentStatus;

pub fn emit_payment_created(
    env: &Env,
    id: u64,
    creator: &Address,
    recipient: &Address,
    amount: i128,
) {
    let topics = (symbol_short!("payment"), symbol_short!("created"));
    env.events().publish(topics, (id, creator.clone(), recipient.clone(), amount));
}

pub fn emit_payment_updated(env: &Env, id: u64, status: PaymentStatus, updated_at: u64) {
    let topics = (symbol_short!("payment"), symbol_short!("updated"));
    env.events().publish(topics, (id, status, updated_at));
}

pub fn emit_payment_completed(env: &Env, id: u64, updated_at: u64) {
    let topics = (symbol_short!("payment"), symbol_short!("completed"));
    env.events().publish(topics, (id, updated_at));
}

pub fn emit_payment_cancelled(env: &Env, id: u64, updated_at: u64) {
    let topics = (symbol_short!("payment"), symbol_short!("cancelled"));
    env.events().publish(topics, (id, updated_at));
}

pub fn emit_role_granted(env: &Env, account: &Address, role: crate::types::Role) {
    let topics = (soroban_sdk::symbol_short!("role"), soroban_sdk::symbol_short!("granted"));
    env.events().publish(topics, (account.clone(), role as u32));
}

pub fn emit_role_revoked(env: &Env, account: &Address, role: crate::types::Role) {
    let topics = (soroban_sdk::symbol_short!("role"), soroban_sdk::symbol_short!("revoked"));
    env.events().publish(topics, (account.clone(), role as u32));
}

pub fn emit_pause_toggled(env: &Env, paused: bool, caller: &Address) {
    let topics = (soroban_sdk::symbol_short!("circuit"), soroban_sdk::symbol_short!("pause"));
    env.events().publish(topics, (paused, caller.clone()));
}

pub fn emit_contract_upgraded(env: &Env, admin: &Address, new_wasm_hash: &soroban_sdk::BytesN<32>) {
    let topics = (soroban_sdk::symbol_short!("contract"), soroban_sdk::symbol_short!("upgraded"));
    env.events().publish(topics, (admin.clone(), new_wasm_hash.clone()));
}

pub fn emit_batch_payments_created(env: &Env, creator: &Address, count: u32) {
    let topics = (soroban_sdk::symbol_short!("batch"), soroban_sdk::symbol_short!("created"));
    env.events().publish(topics, (creator.clone(), count));
}

pub fn emit_memo_hash_bound(env: &Env, id: u64, memo_hash: &soroban_sdk::BytesN<32>) {
    let topics = (soroban_sdk::symbol_short!("memo"), soroban_sdk::symbol_short!("bound"));
    env.events().publish(topics, (id, memo_hash.clone()));
}

pub fn emit_payment_expired(env: &Env, id: u64, timestamp: u64) {
    let topics = (soroban_sdk::symbol_short!("payment"), soroban_sdk::symbol_short!("expired"));
    env.events().publish(topics, (id, timestamp));
}

pub fn emit_dispute_opened(env: &Env, payment_id: u64, initiator: &Address) {
    let topics = (soroban_sdk::symbol_short!("dispute"), soroban_sdk::symbol_short!("opened"));
    env.events().publish(topics, (payment_id, initiator.clone()));
}


pub fn emit_sanctions_hit(env: &Env, account: &Address, reason: &soroban_sdk::String) {
    let topics = (soroban_sdk::symbol_short!("sanction"), account.clone());
    env.events().publish(topics, reason.clone());
}
