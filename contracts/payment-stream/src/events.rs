use soroban_sdk::{symbol_short, Address, Env};
use crate::types::StreamStatus;

pub fn emit_stream_created(env: &Env, id: u64, sender: &Address, recipient: &Address, deposit: i128) {
    let topics = (symbol_short!("stream"), symbol_short!("create"), id);
    env.events().publish(topics, (sender.clone(), recipient.clone(), deposit));
}

pub fn emit_stream_withdrawn(env: &Env, id: u64, recipient: &Address, amount: i128) {
    let topics = (symbol_short!("stream"), symbol_short!("withdraw"), id);
    env.events().publish(topics, (recipient.clone(), amount));
}

pub fn emit_stream_status(env: &Env, id: u64, status: StreamStatus) {
    let topics = (symbol_short!("stream"), symbol_short!("status"), id);
    env.events().publish(topics, status as u32);
}
