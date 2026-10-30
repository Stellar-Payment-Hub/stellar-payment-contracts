use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum StreamStatus {
    Active = 0,
    Paused = 1,
    Cancelled = 2,
    Completed = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamRecord {
    pub id: u64,
    pub sender: Address,
    pub recipient: Address,
    pub token: Address,
    pub deposit_amount: i128,
    pub withdrawn_amount: i128,
    pub start_time: u64,
    pub stop_time: u64,
    pub cliff_time: u64,
    pub rate_per_second: i128,
    pub status: StreamStatus,
    pub paused_at: u64,
    pub pause_duration: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchStreamItem {
    pub recipient: Address,
    pub deposit_amount: i128,
    pub stop_time: u64,
    pub cliff_time: u64,
}
