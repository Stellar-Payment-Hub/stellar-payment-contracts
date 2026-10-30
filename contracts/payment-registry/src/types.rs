use soroban_sdk::{contracttype, Address, BytesN, String};

/// Granular protocol actor roles for access control
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Role {
    Admin = 1,
    Operator = 2,
    Arbitrator = 3,
    Treasury = 4,
}

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
    pub token: Option<Address>,
    pub category: PaymentCategory,
    pub memo: String,
    pub memo_hash: Option<BytesN<32>>,
    pub status: PaymentStatus,
    pub deadline: Option<u64>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Standard categories for classifying on-chain financial settlements
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum PaymentCategory {
    Standard = 0,
    Disbursement = 1,
    Invoice = 2,
    Tip = 3,
    Remittance = 4,
    Escrow = 5,
}

/// Immutable audit entry recording state machine transitions
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentAuditLog {
    pub payment_id: u64,
    pub previous_status: PaymentStatus,
    pub new_status: PaymentStatus,
    pub transitioned_by: Address,
    pub timestamp: u64,
}

/// Dispute status tracking for contested payments
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DisputeStatus {
    None = 0,
    Active = 1,
    ResolvedRefundPayer = 2,
    ResolvedReleaseRecipient = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentDispute {
    pub payment_id: u64,
    pub initiator: Address,
    pub reason: String,
    pub status: DisputeStatus,
    pub opened_at: u64,
    pub resolved_at: Option<u64>,
}
