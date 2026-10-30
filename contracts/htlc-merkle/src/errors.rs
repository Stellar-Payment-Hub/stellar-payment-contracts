use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum HtlcError {
    AlreadyInitialized = 1,
    Unauthorized = 2,
    HtlcNotFound = 3,
    HtlcNotActive = 4,
    InvalidAmount = 5,
    InvalidTimelock = 6,
    TimelockNotExpired = 7,
    TimelockExpired = 8,
    InvalidPreimage = 9,
    DistributionNotFound = 10,
    DistributionExpired = 11,
    DistributionNotExpired = 12,
    InvalidProof = 13,
    LeafAlreadyClaimed = 14,
    SamePartyAddress = 15,
}
