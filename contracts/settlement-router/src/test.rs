#![cfg(test)]
extern crate std;

use super::*;
use payment_registry::{PaymentRegistry, PaymentRegistryClient as RegClient};
use soroban_sdk::{
    testutils::Address as _,
    Address, Env, String, Vec,
};

#[test]
fn test_create_and_execute_settlement_with_inter_contract_call() {
    let env = Env::default();
    env.mock_all_auths();

    // 1. Deploy PaymentRegistry
    let reg_contract_id = env.register(PaymentRegistry, ());
    let reg_client = RegClient::new(&env, &reg_contract_id);
    let admin = Address::generate(&env);
    reg_client.initialize(&admin);

    // 2. Deploy SettlementRouter
    let router_contract_id = env.register(SettlementRouter, ());
    let router_client = SettlementRouterClient::new(&env, &router_contract_id);
    router_client.initialize(&admin, &reg_contract_id);

    // 3. Prepare Multi-Recipient Shares
    let payer = Address::generate(&env);
    let recipient_a = Address::generate(&env);
    let recipient_b = Address::generate(&env);
    let recipient_c = Address::generate(&env);

    let mut shares = Vec::new(&env);
    shares.push_back(RecipientShare {
        recipient: recipient_a.clone(),
        amount: 50_000_000, // 5 XLM
    });
    shares.push_back(RecipientShare {
        recipient: recipient_b.clone(),
        amount: 30_000_000, // 3 XLM
    });
    shares.push_back(RecipientShare {
        recipient: recipient_c.clone(),
        amount: 20_000_000, // 2 XLM
    });

    let memo = String::from_str(&env, "Group Dinner Settlement");

    // 4. Create Settlement
    let settlement_id = router_client.create_settlement(&payer, &shares, &memo);
    assert_eq!(settlement_id, 1);

    let record = router_client.get_settlement(&settlement_id);
    assert_eq!(record.total_amount, 100_000_000);
    assert_eq!(record.recipient_count, 3);
    assert_eq!(record.status, SettlementStatus::Created);

    // 5. Execute Settlement (Triggers inter-contract calls into PaymentRegistry)
    let executed_id = router_client.execute_settlement(&settlement_id, &payer);
    assert_eq!(executed_id, 1);

    let updated = router_client.get_settlement(&settlement_id);
    assert_eq!(updated.status, SettlementStatus::Completed);
    assert_eq!(updated.sub_payment_ids.len(), 3);

    // 6. Verify that PaymentRegistry received the 3 sub-payments!
    assert_eq!(reg_client.get_payment_count(), 3);
    let p1 = reg_client.get_payment(&1);
    assert_eq!(p1.creator, payer);
    assert_eq!(p1.recipient, recipient_a);
    assert_eq!(p1.amount, 50_000_000);

    let p2 = reg_client.get_payment(&2);
    assert_eq!(p2.recipient, recipient_b);
    assert_eq!(p2.amount, 30_000_000);
}

#[test]
fn test_split_bill_equal_shares() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let reg_contract_id = env.register(PaymentRegistry, ());
    let reg_client = RegClient::new(&env, &reg_contract_id);
    reg_client.initialize(&admin);

    let router_contract_id = env.register(SettlementRouter, ());
    let router_client = SettlementRouterClient::new(&env, &router_contract_id);
    router_client.initialize(&admin, &reg_contract_id);

    let payer = Address::generate(&env);
    let r1 = Address::generate(&env);
    let r2 = Address::generate(&env);
    let r3 = Address::generate(&env);
    let r4 = Address::generate(&env);

    let mut recipients = Vec::new(&env);
    recipients.push_back(r1);
    recipients.push_back(r2);
    recipients.push_back(r3);
    recipients.push_back(r4);

    let memo = String::from_str(&env, "Lunch Split");
    let total = 120_000_000; // 12 XLM, 4 people = 3 XLM each

    let sid = router_client.split_bill(&payer, &total, &recipients, &memo);
    assert_eq!(sid, 1);

    let record = router_client.get_settlement(&sid);
    assert_eq!(record.total_amount, 120_000_000);
    assert_eq!(record.recipient_count, 4);

    let shares = router_client.get_recipients(&sid);
    for i in 0..4 {
        assert_eq!(shares.get(i).unwrap().amount, 30_000_000);
    }
}

#[test]
fn test_split_bill_remainder_allocation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let reg_id = env.register(PaymentRegistry, ());
    let router_id = env.register(SettlementRouter, ());
    let router = SettlementRouterClient::new(&env, &router_id);
    router.initialize(&admin, &reg_id);

    let payer = Address::generate(&env);
    let mut recipients = Vec::new(&env);
    recipients.push_back(Address::generate(&env));
    recipients.push_back(Address::generate(&env));
    recipients.push_back(Address::generate(&env));

    let total = 100; // 100 / 3 = 33, 33, 34
    let sid = router.split_bill(&payer, &total, &recipients, &String::from_str(&env, "Remainder"));

    let shares = router.get_recipients(&sid);
    assert_eq!(shares.get(0).unwrap().amount, 33);
    assert_eq!(shares.get(1).unwrap().amount, 33);
    assert_eq!(shares.get(2).unwrap().amount, 34);
}

#[test]
fn test_cancel_settlement() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let reg_id = env.register(PaymentRegistry, ());
    let router_id = env.register(SettlementRouter, ());
    let router = SettlementRouterClient::new(&env, &router_id);
    router.initialize(&admin, &reg_id);

    let payer = Address::generate(&env);
    let mut shares = Vec::new(&env);
    shares.push_back(RecipientShare {
        recipient: Address::generate(&env),
        amount: 10_000_000,
    });

    let sid = router.create_settlement(&payer, &shares, &String::from_str(&env, "Cancel me"));
    router.cancel_settlement(&sid, &payer);

    let record = router.get_settlement(&sid);
    assert_eq!(record.status, SettlementStatus::Cancelled);
}

#[test]
fn test_empty_recipients_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let reg_id = env.register(PaymentRegistry, ());
    let router_id = env.register(SettlementRouter, ());
    let router = SettlementRouterClient::new(&env, &router_id);
    router.initialize(&admin, &reg_id);

    let payer = Address::generate(&env);
    let empty_shares = Vec::new(&env);

    let res = router.try_create_settlement(&payer, &empty_shares, &String::from_str(&env, "Empty"));
    assert!(res.is_err());
}

#[test]
fn test_unauthorized_execution_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let reg_id = env.register(PaymentRegistry, ());
    let router_id = env.register(SettlementRouter, ());
    let router = SettlementRouterClient::new(&env, &router_id);
    router.initialize(&admin, &reg_id);

    let payer = Address::generate(&env);
    let imposter = Address::generate(&env);
    let mut shares = Vec::new(&env);
    shares.push_back(RecipientShare {
        recipient: Address::generate(&env),
        amount: 10_000_000,
    });

    let sid = router.create_settlement(&payer, &shares, &String::from_str(&env, "Auth test"));
    let res = router.try_execute_settlement(&sid, &imposter);
    assert!(res.is_err());
}

