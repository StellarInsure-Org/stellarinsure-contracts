use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyType {
    Weather,
    SmartContract,
    Flight,
    Health,
    Asset,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyStatus {
    Active,
    Expired,
    Cancelled,
    ClaimPending,
    ClaimApproved,
    ClaimRejected,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Policy {
    pub id: u64,
    pub policyholder: Address,
    pub policy_type: PolicyType,
    pub coverage_amount: i128,
    pub premium: i128,
    pub start_time: u64,
    pub end_time: u64,
    pub trigger_condition: String,
    pub status: PolicyStatus,
    pub claim_amount: i128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Claim {
    pub policy_id: u64,
    pub claim_amount: i128,
    pub proof: String,
    pub timestamp: u64,
    pub approved: bool,
}
