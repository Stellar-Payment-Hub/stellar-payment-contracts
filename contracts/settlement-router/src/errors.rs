use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SettlementError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    SettlementNotFound = 4,
    InvalidStatusTransition = 5,
    InvalidTotalAmount = 6,
    EmptyRecipients = 7,
    TooManyRecipients = 8,
    InvalidShareAmount = 9,
    PayerIsRecipient = 10,
    SumMismatch = 11,
    RegistryCallFailed = 12,
    ContractPaused = 13,
    InvalidFeeBps = 14,
    ReentrancyGuard = 15,
    TreasuryNotConfigured = 16,
    DisputeActive = 17,
    InvalidWeight = 18,
    ZeroWeightSum = 19,
    DeadlineExpired = 20,
    DeadlineNotPassed = 21,
    MilestoneNotFound = 22,
    MilestoneAlreadyExecuted = 23,
    InsufficientAllowance = 24,
    InsufficientTokenBalance = 25,
    TokenNotWhitelisted = 26,
    ZeroTokenAmount = 27,
    FeeExceedsCap = 28,
    TokenTransferFailed = 29,
    ContractReentrancy = 30,
}
