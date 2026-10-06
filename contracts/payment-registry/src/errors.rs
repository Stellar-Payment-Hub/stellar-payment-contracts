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
}
