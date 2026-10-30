#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger as _}, Address, Env};

#[test]
fn test_payment_stream_accrual_and_withdrawal() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(PaymentStream, ());
    let client = PaymentStreamClient::new(&env, &contract_id);
    client.initialize(&admin);

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token = Address::generate(&env);

    // 1. Create stream: 100 XLM over 1000 seconds (100_000 stroops/sec), 0 cliff
    let stream_id = client.create_stream(&sender, &recipient, &token, &100_000_000, &1000, &0);
    assert_eq!(stream_id, 1);

    let stream = client.get_stream(&stream_id).unwrap();
    assert_eq!(stream.rate_per_second, 100_000);

    // Immediately after creation: 0 withdrawable
    assert_eq!(client.calculate_withdrawable(&stream_id), 0);

    // Advance ledger by 250 seconds
    let now = env.ledger().timestamp();
    env.ledger().set_timestamp(now + 250);

    // Should have accrued 25 XLM (25_000_000)
    let withdrawable = client.calculate_withdrawable(&stream_id);
    assert_eq!(withdrawable, 25_000_000);

    // Recipient claims 25 XLM
    let claimed = client.withdraw_from_stream(&recipient, &stream_id);
    assert_eq!(claimed, 25_000_000);

    // Check stream balance breakdown
    let (claimed_total, avail, locked) = client.get_stream_balances(&stream_id);
    assert_eq!(claimed_total, 25_000_000);
    assert_eq!(avail, 0);
    assert_eq!(locked, 75_000_000);

    // Advance past stop time
    env.ledger().set_timestamp(now + 1050);
    let final_withdrawable = client.calculate_withdrawable(&stream_id);
    assert_eq!(final_withdrawable, 75_000_000);

    // Final claim -> status Completed
    client.withdraw_from_stream(&recipient, &stream_id);
    let final_stream = client.get_stream(&stream_id).unwrap();
    assert_eq!(final_stream.status, StreamStatus::Completed);
}

#[test]
fn test_stream_cliff_and_cancellation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(PaymentStream, ());
    let client = PaymentStreamClient::new(&env, &contract_id);
    client.initialize(&admin);

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token = Address::generate(&env);

    // 100 XLM over 1000 seconds, with 500 second cliff
    let stream_id = client.create_stream(&sender, &recipient, &token, &100_000_000, &1000, &500);
    assert_eq!(client.is_cliff_passed(&stream_id), false);

    // Advance by 300s (before cliff): withdrawable must be 0
    let now = env.ledger().timestamp();
    env.ledger().set_timestamp(now + 300);
    assert_eq!(client.calculate_withdrawable(&stream_id), 0);

    // Advance to 600s (after cliff): step jump to 60 XLM (60_000_000)
    env.ledger().set_timestamp(now + 600);
    assert_eq!(client.is_cliff_passed(&stream_id), true);
    assert_eq!(client.calculate_withdrawable(&stream_id), 60_000_000);

    // Sender cancels: 60 XLM to recipient, 40 XLM refunded to sender
    let (to_rec, to_send) = client.cancel_by_sender(&sender, &stream_id);
    assert_eq!(to_rec, 60_000_000);
    assert_eq!(to_send, 40_000_000);

    let cancelled = client.get_stream(&stream_id).unwrap();
    assert_eq!(cancelled.status, StreamStatus::Cancelled);
}
