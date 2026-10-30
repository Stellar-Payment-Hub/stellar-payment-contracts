use soroban_sdk::{symbol_short, Address, BytesN, Env};

pub fn emit_htlc_created(env: &Env, id: u64, sender: &Address, recipient: &Address, amount: i128) {
    let topics = (symbol_short!("htlc"), symbol_short!("created"), id);
    env.events()
        .publish(topics, (sender.clone(), recipient.clone(), amount));
}

pub fn emit_htlc_claimed(env: &Env, id: u64, recipient: &Address, preimage: &BytesN<32>) {
    let topics = (symbol_short!("htlc"), symbol_short!("claimed"), id);
    env.events()
        .publish(topics, (recipient.clone(), preimage.clone()));
}

pub fn emit_htlc_refunded(env: &Env, id: u64, sender: &Address) {
    let topics = (symbol_short!("htlc"), symbol_short!("refunded"), id);
    env.events().publish(topics, sender.clone());
}

pub fn emit_merkle_claimed(env: &Env, dist_id: u64, index: u32, recipient: &Address, amount: i128) {
    let topics = (symbol_short!("merkle"), dist_id, index);
    env.events().publish(topics, (recipient.clone(), amount));
}
