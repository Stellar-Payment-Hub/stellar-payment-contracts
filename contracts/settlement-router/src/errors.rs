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
}
