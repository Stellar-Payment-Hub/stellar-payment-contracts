use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StreamError {
    AlreadyInitialized = 1,
    Unauthorized = 2,
    StreamNotFound = 3,
    StreamNotActive = 4,
    InvalidAmount = 5,
    InvalidDuration = 6,
    CliffPastStopTime = 7,
    CliffNotReached = 8,
    ZeroWithdrawable = 9,
    SamePartyAddress = 10,
    ContractPaused = 11,
}
