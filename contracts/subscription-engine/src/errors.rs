use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SubscriptionError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    PlanNotFound = 4,
    SubscriptionNotFound = 5,
    PlanInactive = 6,
    InvalidInterval = 7,
    InvalidAmount = 8,
    BillingEpochNotDue = 9,
    MaxCyclesReached = 10,
    SubscriptionPaused = 11,
    SubscriptionCancelled = 12,
    ContractPaused = 13,
    SameAddress = 14,
}
