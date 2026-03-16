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
}
