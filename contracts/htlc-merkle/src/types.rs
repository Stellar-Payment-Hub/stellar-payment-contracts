use soroban_sdk::{contracttype, Address, BytesN};

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum HtlcStatus {
    Active = 0,
    Claimed = 1,
    Refunded = 2,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtlcRecord {
    pub id: u64,
    pub sender: Address,
    pub recipient: Address,
    pub token: Address,
    pub amount: i128,
    pub hashlock: BytesN<32>,
    pub timelock: u64,
    pub status: HtlcStatus,
    pub preimage: Option<BytesN<32>>,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerkleDistribution {
    pub id: u64,
    pub distributor: Address,
    pub token: Address,
    pub root: BytesN<32>,
    pub total_amount: i128,
    pub remaining_amount: i128,
    pub expires_at: u64,
    pub claims_count: u32,
}
