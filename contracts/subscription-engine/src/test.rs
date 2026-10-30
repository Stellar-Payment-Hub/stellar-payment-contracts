#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger as _}, Address, Env, String};

#[test]
fn test_subscription_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(SubscriptionEngine, ());
    let client = SubscriptionEngineClient::new(&env, &contract_id);
    client.initialize(&admin);

    let merchant = Address::generate(&env);
    let subscriber = Address::generate(&env);
    let token = Address::generate(&env);
    let plan_name = String::from_str(&env, "Pro Monthly");

    // 1. Create plan: 20 XLM every 30 days (2,592,000s), max 12 cycles
    let plan_id = client.create_plan(&merchant, &token, &plan_name, &20_000_000, &2_592_000, &12);
    assert_eq!(plan_id, 1);

    let plan = client.get_plan(&plan_id).unwrap();
    assert_eq!(plan.amount, 20_000_000);
    assert_eq!(plan.interval_seconds, 2_592_000);

    // 2. Subscribe
    let sub_id = client.subscribe(&subscriber, &plan_id);
    assert_eq!(sub_id, 1);
    assert_eq!(client.get_subscriber_count(&merchant), 1);
    assert_eq!(client.get_merchant_mrr(&merchant), 20_000_000);

    // 3. Charge initially (due at timestamp)
    assert_eq!(client.is_charge_due(&sub_id), true);
    let cycle = client.charge_subscriber(&merchant, &sub_id);
    assert_eq!(cycle, 1);

    // Cannot charge again immediately
    assert_eq!(client.is_charge_due(&sub_id), false);
    let premature = client.try_charge_subscriber(&merchant, &sub_id);
    assert!(premature.is_err());

    // Fast forward ledger time by 30 days
    let now = env.ledger().timestamp();
    env.ledger().set_timestamp(now + 2_592_000);
    assert_eq!(client.is_charge_due(&sub_id), true);

    // Charge cycle 2
    let cycle2 = client.charge_subscriber(&merchant, &sub_id);
    assert_eq!(cycle2, 2);

    let sub = client.get_subscription(&sub_id).unwrap();
    assert_eq!(sub.total_paid, 40_000_000);
    assert_eq!(sub.cycles_billed, 2);

    // 4. Pause and attempt charge (should fail)
    client.pause_subscription(&subscriber, &sub_id);
    let paused_charge = client.try_charge_subscriber(&merchant, &sub_id);
    assert!(paused_charge.is_err());

    // Resume
    client.resume_subscription(&subscriber, &sub_id);

    // 5. Cancel by subscriber
    client.cancel_by_subscriber(&subscriber, &sub_id);
    let cancelled_sub = client.get_subscription(&sub_id).unwrap();
    assert_eq!(cancelled_sub.status, SubscriptionStatus::Cancelled);
    assert_eq!(client.get_subscriber_count(&merchant), 0);
    assert_eq!(client.get_merchant_mrr(&merchant), 0);
}
