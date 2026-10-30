#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger as _}, Address, Bytes, BytesN, Env, Vec};

#[test]
fn test_htlc_preimage_claim_and_refund() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(HtlcMerkleVault, ());
    let client = HtlcMerkleVaultClient::new(&env, &contract_id);
    client.initialize(&admin);

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token = Address::generate(&env);

    // 1. Create preimage and calculate hashlock
    let preimage = BytesN::from_array(&env, &[42u8; 32]);
    let hashlock = client.hash_preimage(&preimage);

    // 2. Create HTLC for 50 XLM, 2 hour timelock (7200s)
    let htlc_id = client.create_htlc(&sender, &recipient, &token, &50_000_000, &hashlock, &7200);
    assert_eq!(htlc_id, 1);

    let record = client.get_htlc(&htlc_id).unwrap();
    assert_eq!(record.amount, 50_000_000);
    assert_eq!(record.status, HtlcStatus::Active);

    // 3. Claim with wrong preimage -> fails
    let wrong_preimage = BytesN::from_array(&env, &[99u8; 32]);
    let failed_claim = client.try_claim_htlc(&recipient, &htlc_id, &wrong_preimage);
    assert!(failed_claim.is_err());

    // 4. Claim with correct preimage -> succeeds
    let claimed_amount = client.claim_htlc(&recipient, &htlc_id, &preimage);
    assert_eq!(claimed_amount, 50_000_000);

    let final_record = client.get_htlc(&htlc_id).unwrap();
    assert_eq!(final_record.status, HtlcStatus::Claimed);
    assert_eq!(final_record.preimage, Some(preimage));
}

#[test]
fn test_htlc_timeout_refund() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(HtlcMerkleVault, ());
    let client = HtlcMerkleVaultClient::new(&env, &contract_id);
    client.initialize(&admin);

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token = Address::generate(&env);

    let preimage = BytesN::from_array(&env, &[15u8; 32]);
    let hashlock = client.hash_preimage(&preimage);

    let htlc_id = client.create_htlc(&sender, &recipient, &token, &30_000_000, &hashlock, &3600);

    // Attempt refund before timelock -> fails
    let early_refund = client.try_refund_htlc(&sender, &htlc_id);
    assert!(early_refund.is_err());

    // Advance ledger past timelock (3601s)
    let now = env.ledger().timestamp();
    env.ledger().set_timestamp(now + 3605);

    // Refund succeeds
    let refunded = client.refund_htlc(&sender, &htlc_id);
    assert_eq!(refunded, 30_000_000);

    let record = client.get_htlc(&htlc_id).unwrap();
    assert_eq!(record.status, HtlcStatus::Refunded);
}

#[test]
fn test_merkle_distribution_and_claim() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(HtlcMerkleVault, ());
    let client = HtlcMerkleVaultClient::new(&env, &contract_id);
    client.initialize(&admin);

    let distributor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token = Address::generate(&env);

    // Single-leaf Merkle root = sha256(0 || 10_000_000)
    let mut leaf_bytes = Bytes::new(&env);
    leaf_bytes.append(&Bytes::from_slice(&env, &0u32.to_be_bytes()));
    leaf_bytes.append(&Bytes::from_slice(&env, &10_000_000i128.to_be_bytes()));
    let root: BytesN<32> = env.crypto().sha256(&leaf_bytes).into();

    // Register distribution
    let dist_id = client.register_merkle_dist(&distributor, &token, &root, &10_000_000, &86400);
    assert_eq!(dist_id, 1);
    assert_eq!(client.is_leaf_claimed(&dist_id, &0), false);

    // Claim with empty proof (since leaf == root)
    let proof = Vec::new(&env);
    let claimed = client.claim_merkle_leaf(&recipient, &dist_id, &0, &10_000_000, &proof);
    assert_eq!(claimed, 10_000_000);
    assert_eq!(client.is_leaf_claimed(&dist_id, &0), true);

    // Double-claim fails
    let double_claim = client.try_claim_merkle_leaf(&recipient, &dist_id, &0, &10_000_000, &proof);
    assert!(double_claim.is_err());
}

#[test]
fn test_multihop_atomic_swap_invariant() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(HtlcMerkleVault, ());
    let client = HtlcMerkleVaultClient::new(&env, &contract_id);
    client.initialize(&admin);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let usdc = Address::generate(&env);
    let xlm = Address::generate(&env);

    // Alice generates secret
    let secret = BytesN::from_array(&env, &[88u8; 32]);
    let hashlock = client.hash_preimage(&secret);

    // Hop 1: Alice locks 100 USDC for Bob with 2h timelock
    let leg1 = client.create_htlc(&alice, &bob, &usdc, &100_000_000, &hashlock, &7200);

    // Hop 2: Bob locks 800 XLM for Alice with 1h timelock (Bob has shorter timelock)
    let leg2 = client.create_htlc(&bob, &alice, &xlm, &800_000_000, &hashlock, &3600);

    // Alice claims Hop 2 by revealing secret
    let alice_claimed = client.claim_htlc(&alice, &leg2, &secret);
    assert_eq!(alice_claimed, 800_000_000);

    // The secret is now revealed in leg2's public state!
    let revealed_secret = client.get_htlc(&leg2).unwrap().preimage.unwrap();
    assert_eq!(revealed_secret, secret);

    // Bob reads the secret and claims Hop 1 before the 2h timelock expires
    let bob_claimed = client.claim_htlc(&bob, &leg1, &revealed_secret);
    assert_eq!(bob_claimed, 100_000_000);

    // Both legs are settled symmetrically
    assert_eq!(client.get_htlc(&leg1).unwrap().status, HtlcStatus::Claimed);
    assert_eq!(client.get_htlc(&leg2).unwrap().status, HtlcStatus::Claimed);
}
