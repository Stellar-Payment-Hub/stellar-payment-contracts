use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum EscrowError {
    AlreadyInitialized = 1,
    Unauthorized = 2,
    EscrowNotFound = 3,
    EscrowNotActive = 4,
    InvalidAmount = 5,
    InvalidDeadline = 6,
    SamePartyAddress = 7,
    InvalidSplitSum = 8,
    DeadlineNotReached = 9,
    MilestoneNotFound = 10,
    MilestoneAlreadyApproved = 11,
    FeeExceedsCap = 12,
}
