use soroban_sdk::{contracttype, Address, String, Vec};

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SettlementStatus {
    Created = 0,
    Processing = 1,
    Completed = 2,
    Cancelled = 3,
    Failed = 4,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipientShare {
    pub recipient: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlementRecord {
    pub id: u64,
    pub payer: Address,
    pub total_amount: i128,
    pub recipient_count: u32,
    pub memo: String,
    pub status: SettlementStatus,
    pub sub_payment_ids: Vec<u64>,
    pub created_at: u64,
    pub updated_at: u64,
}
