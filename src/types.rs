use soroban_sdk::{contracttype, Address, String, Vec};

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyCreatedEvent {
    pub policy_id: u64,
    pub policyholder: Address,
    pub policy_type: PolicyType,
    pub coverage_amount: i128,
    pub premium: i128,
    pub start_time: u64,
    pub end_time: u64,
    pub trigger_condition: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PremiumPaidEvent {
    pub policy_id: u64,
    pub policyholder: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimSubmittedEvent {
    pub policy_id: u64,
    pub policyholder: Address,
    pub claim_amount: i128,
    pub proof: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimProcessedEvent {
    pub policy_id: u64,
    pub policyholder: Address,
    pub claim_amount: i128,
    pub approved: bool,
    pub status: PolicyStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyCancelledEvent {
    pub policy_id: u64,
    pub policyholder: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderPosition {
    pub provider: Address,
    pub contribution: i128,
    pub accrued_yield: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolStats {
    pub total_liquidity: i128,
    pub total_yield_distributed: i128,
    pub provider_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YieldDistributionEvent {
    pub amount: i128,
    pub total_liquidity_before_distribution: i128,
    pub provider_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiquidityAddedEvent {
    pub provider: Address,
    pub amount: i128,
    pub new_contribution: i128,
    pub pool_balance: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiquidityWithdrawnEvent {
    pub provider: Address,
    pub amount: i128,
    pub remaining_contribution: i128,
    pub pool_balance: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YieldClaimedEvent {
    pub provider: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Providers(pub Vec<Address>);
