use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum PaymentStatus {
    Pending = 0,
    Processing = 1,
    Completed = 2,
    Failed = 3,
    Cancelled = 4,
    Expired = 5,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Payment {
    pub id: u64,
    pub creator: Address,
    pub recipient: Address,
    pub amount: i128,
    pub memo: String,
    pub status: PaymentStatus,
    pub created_at: u64,
    pub updated_at: u64,
}
