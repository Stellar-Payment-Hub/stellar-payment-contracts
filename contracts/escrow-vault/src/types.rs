use soroban_sdk::{contracttype, Address, BytesN, String};

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum EscrowStatus {
    Pending = 0,
    Funded = 1,
    Disputed = 2,
    Released = 3,
    Refunded = 4,
    PartiallySettled = 5,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum VoteDecision {
    None = 0,
    ReleaseToSeller = 1,
    RefundToBuyer = 2,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowRecord {
    pub id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub arbitrator: Address,
    pub token: Address,
    pub total_amount: i128,
    pub remaining_amount: i128,
    pub arbitrator_fee_bps: u32,
    pub deadline: u64,
    pub status: EscrowStatus,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowMilestone {
    pub escrow_id: u64,
    pub milestone_index: u32,
    pub amount: i128,
    pub description: String,
    pub evidence_hash: Option<BytesN<32>>,
    pub approved: bool,
}
