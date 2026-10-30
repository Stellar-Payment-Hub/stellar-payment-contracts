use soroban_sdk::{contracttype, Address, BytesN, String, Vec};

/// Configurable fee parameters in basis points (1 bps = 0.01%)
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FeeConfig {
    pub fee_bps: u32,       // e.g. 50 = 0.5%
    pub max_fee_bps: u32,   // hard cap e.g. 500 = 5%
    pub enabled: bool,
}



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
    pub token: Option<Address>,
    pub total_amount: i128,
    pub fee_amount: i128,
    pub recipient_count: u32,
    pub memo: String,
    pub memo_hash: Option<BytesN<32>>,
    pub status: SettlementStatus,
    pub sub_payment_ids: Vec<u64>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Weighted split parameter specifying fractional share by relative weight
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeightedShare {
    pub recipient: Address,
    pub weight: u32,
}

/// Milestone payment stage for progressive project disbursements
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlementMilestone {
    pub settlement_id: u64,
    pub milestone_index: u32,
    pub description: String,
    pub amount: i128,
    pub executed: bool,
}

/// Timelock parameters for conditional settlement escrow
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct EscrowTimelock {
    pub release_after: u64,
    pub auto_refund_after: u64,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DisputeVerdict {
    Pending = 0,
    RefundPayer = 1,
    ReleaseRecipients = 2,
    SplitEvenly = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlementDispute {
    pub settlement_id: u64,
    pub initiator: Address,
    pub evidence_hash: BytesN<32>,
    pub verdict: DisputeVerdict,
    pub initiated_at: u64,
    pub resolved_at: Option<u64>,
}
