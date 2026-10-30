use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SubscriptionStatus {
    Active = 0,
    Paused = 1,
    Cancelled = 2,
    Expired = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionPlan {
    pub id: u64,
    pub merchant: Address,
    pub token: Address,
    pub name: String,
    pub amount: i128,
    pub interval_seconds: u64,
    pub max_cycles: u32,
    pub active: bool,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionRecord {
    pub id: u64,
    pub plan_id: u64,
    pub subscriber: Address,
    pub merchant: Address,
    pub token: Address,
    pub status: SubscriptionStatus,
    pub started_at: u64,
    pub last_billed_at: u64,
    pub next_bill_at: u64,
    pub cycles_billed: u32,
    pub max_cycles: u32,
    pub total_paid: i128,
}
