use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidAmount = 1,
    InvalidPremium = 2,
    PolicyNotFound = 3,
    PolicyNotActive = 4,
    PolicyExpired = 5,
    ClaimExceedsCoverage = 6,
    NoPendingClaim = 7,
    Unauthorized = 8,
    ClaimNotFound = 9,
    AlreadyInitialized = 10,
    InvalidDuration = 11,
    InvalidClaimAmount = 12,
    InsufficientLiquidity = 13,
    ProviderNotFound = 14,
    NoYieldAvailable = 15,
    NotInitialized = 16,
    ContractPaused = 17,
    // Issue #16 — multi-sig admin
    AdminAlreadyExists = 18,
    AdminNotFound = 19,
    InvalidThreshold = 20,
    AlreadyVoted = 21,
    // Issue #22 — policy renewal
    RenewalGracePeriodExpired = 22,
    PolicyNotRenewable = 23,
    InsufficientContractBalance = 24,
    // Oracle Integration Stub
    OracleVerificationFailed = 25,
}
