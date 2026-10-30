use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    PaymentNotFound = 2,
    InvalidAmount = 3,
    InvalidStatus = 4,
    AlreadyCompleted = 5,
    AlreadyCancelled = 6,
    Unauthorized = 7,
    InvalidTransition = 8,
    SameAddress = 9,
    ContractPaused = 10,
    ReentrancyGuard = 11,
    PaymentExpired = 12,
    DisputeAlreadyActive = 13,
    DisputeNotFound = 14,
    InvalidCategory = 15,
    InvalidDeadline = 16,
    BatchEmpty = 17,
    BatchTooLarge = 18,
    RoleRequired = 19,
    InvalidMemoHash = 20,
}
