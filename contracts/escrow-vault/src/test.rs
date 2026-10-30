#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

#[test]
fn test_escrow_multisig_and_split_arbitration() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(EscrowVault, ());
    let client = EscrowVaultClient::new(&env, &contract_id);
    client.initialize(&admin);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let token = Address::generate(&env);

    // 1. Create escrow for 100 XLM, 2% (200 bps) arb fee, 7 days (604,800s)
    let escrow_id = client.create_escrow(
        &buyer,
        &seller,
        &arbitrator,
        &token,
        &100_000_000,
        &200,
        &604_800,
    );
    assert_eq!(escrow_id, 1);

    let record = client.get_escrow(&escrow_id).unwrap();
    assert_eq!(record.total_amount, 100_000_000);
    assert_eq!(record.status, EscrowStatus::Funded);

    // 2. Raise dispute
    client.raise_dispute(&buyer, &escrow_id);
    let disputed = client.get_escrow(&escrow_id).unwrap();
    assert_eq!(disputed.status, EscrowStatus::Disputed);

    // 3. Resolve with split award (arbitrator fee is 2% of 100 XLM = 2 XLM = 2_000_000)
    // Distributable = 98_000_000. Let's split 60 XLM to buyer (60_000_000) and 38 XLM to seller (38_000_000)
    let fee = client.resolve_split_award(&arbitrator, &escrow_id, &60_000_000, &38_000_000);
    assert_eq!(fee, 2_000_000);

    let settled = client.get_escrow(&escrow_id).unwrap();
    assert_eq!(settled.status, EscrowStatus::PartiallySettled);
    assert_eq!(settled.remaining_amount, 0);
}

#[test]
fn test_milestone_progressive_workflow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(EscrowVault, ());
    let client = EscrowVaultClient::new(&env, &contract_id);
    client.initialize(&admin);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let token = Address::generate(&env);

    let escrow_id = client.create_escrow(
        &buyer,
        &seller,
        &arbitrator,
        &token,
        &50_000_000,
        &0,
        &604_800,
    );

    // Add milestone 1 (20 XLM) and milestone 2 (30 XLM)
    let desc1 = String::from_str(&env, "Design Mockups");
    let desc2 = String::from_str(&env, "Production Code");
    client.add_escrow_milestone(&buyer, &escrow_id, &1, &20_000_000, &desc1);
    client.add_escrow_milestone(&buyer, &escrow_id, &2, &30_000_000, &desc2);

    // Seller submits evidence hash for milestone 1
    let hash = BytesN::from_array(&env, &[7u8; 32]);
    client.submit_milestone_evidence(&seller, &escrow_id, &1, &hash);

    // Buyer approves milestone 1
    let released = client.approve_milestone(&buyer, &escrow_id, &1);
    assert_eq!(released, 20_000_000);

    let esc = client.get_escrow(&escrow_id).unwrap();
    assert_eq!(esc.remaining_amount, 30_000_000);
    assert_eq!(esc.status, EscrowStatus::Funded);

    // Buyer approves milestone 2 -> transitions to Released
    client.approve_milestone(&buyer, &escrow_id, &2);
    let final_esc = client.get_escrow(&escrow_id).unwrap();
    assert_eq!(final_esc.remaining_amount, 0);
    assert_eq!(final_esc.status, EscrowStatus::Released);
}
