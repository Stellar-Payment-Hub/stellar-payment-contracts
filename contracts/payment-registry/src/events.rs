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
