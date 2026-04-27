use soroban_sdk::{contracttype, Address, Env, Vec};

use crate::{Claim, ClaimVotes, Error, Policy, PoolStats, ProviderPosition, Providers};

#[contracttype]
enum DataKey {
    Admin,
    PolicyCounter,
    Policy(u64),
    Claim(u64),
    PremiumToken,
    RiskPoolAdmin,
    TotalLiquidity,
    TotalYieldDistributed,
    Provider(Address),
    Providers,
    Paused,
    // Issue #16 — multi-sig
    Admins,
    Threshold,
    ClaimVotes(u64),
    Version,
    TotalPremium,
    TotalPayouts,
    RiskPool,
    // Issue #199 — max policies
    MaxPolicies,
    // Issue #202 — reserve ratio
    ReserveRatio,
}

pub fn get_version(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::Version).unwrap_or(1)
}

pub fn set_version(env: &Env, version: u32) {
    env.storage().instance().set(&DataKey::Version, &version);
}

fn policy_key(policy_id: u64) -> DataKey {
    DataKey::Policy(policy_id)
}

fn claim_key(policy_id: u64) -> DataKey {
    DataKey::Claim(policy_id)
}

fn provider_key(provider: &Address) -> DataKey {
    DataKey::Provider(provider.clone())
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn set_policy_counter(env: &Env, counter: u64) {
    env.storage()
        .instance()
        .set(&DataKey::PolicyCounter, &counter);
}

pub fn get_policy_counter(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::PolicyCounter)
        .unwrap_or(0)
}

pub fn set_policy(env: &Env, policy_id: u64, policy: &Policy) {
    env.storage()
        .persistent()
        .set(&policy_key(policy_id), policy);
}

pub fn get_policy(env: &Env, policy_id: u64) -> Result<Policy, Error> {
    env.storage()
        .persistent()
        .get(&policy_key(policy_id))
        .ok_or(Error::PolicyNotFound)
}

pub fn set_claim(env: &Env, policy_id: u64, claim: &Claim) {
    env.storage().persistent().set(&claim_key(policy_id), claim);
}

pub fn get_claim(env: &Env, policy_id: u64) -> Result<Claim, Error> {
    env.storage()
        .persistent()
        .get(&claim_key(policy_id))
        .ok_or(Error::ClaimNotFound)
}

pub fn has_risk_pool_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::RiskPoolAdmin)
}

pub fn set_risk_pool_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::RiskPoolAdmin, admin);
}

pub fn get_risk_pool_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::RiskPoolAdmin)
        .unwrap()
}

pub fn set_total_liquidity(env: &Env, amount: i128) {
    env.storage()
        .instance()
        .set(&DataKey::TotalLiquidity, &amount);
}

pub fn get_total_liquidity(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalLiquidity)
        .unwrap_or(0)
}

pub fn set_total_yield_distributed(env: &Env, amount: i128) {
    env.storage()
        .instance()
        .set(&DataKey::TotalYieldDistributed, &amount);
}

pub fn get_total_yield_distributed(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalYieldDistributed)
        .unwrap_or(0)
}

pub fn get_provider(env: &Env, provider: &Address) -> Option<ProviderPosition> {
    env.storage().persistent().get(&provider_key(provider))
}

pub fn set_provider(env: &Env, provider: &Address, position: &ProviderPosition) {
    env.storage()
        .persistent()
        .set(&provider_key(provider), position);
}

pub fn remove_provider(env: &Env, provider: &Address) {
    env.storage().persistent().remove(&provider_key(provider));
}

pub fn get_providers(env: &Env) -> Vec<Address> {
    env.storage()
        .instance()
        .get::<_, Providers>(&DataKey::Providers)
        .map(|providers| providers.0)
        .unwrap_or(Vec::new(env))
}

pub fn set_providers(env: &Env, providers: &Vec<Address>) {
    env.storage()
        .instance()
        .set(&DataKey::Providers, &Providers(providers.clone()));
}

pub fn ensure_provider_registered(env: &Env, provider: &Address) {
    let mut providers = get_registered_provider_vec(env);
    let mut already_registered = false;
    for candidate in providers.iter() {
        if candidate == *provider {
            already_registered = true;
            break;
        }
    }

    if !already_registered {
        providers.push_back(provider.clone());
        env.storage()
            .instance()
            .set(&DataKey::Providers, &Providers(providers));
    }
}

pub fn unregister_provider(env: &Env, provider: &Address) {
    let providers = get_registered_provider_vec(env);
    let mut filtered = Vec::new(env);

    for candidate in providers.iter() {
        if candidate != *provider {
            filtered.push_back(candidate);
        }
    }

    env.storage()
        .instance()
        .set(&DataKey::Providers, &Providers(filtered));
}

pub fn get_registered_provider_vec(env: &Env) -> Vec<Address> {
    env.storage()
        .instance()
        .get::<_, Providers>(&DataKey::Providers)
        .map(|providers| providers.0)
        .unwrap_or(Vec::new(env))
}

pub fn get_premium_token(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::PremiumToken)
}

pub fn set_premium_token(env: &Env, token: &Address) {
    env.storage().instance().set(&DataKey::PremiumToken, token);
}

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&DataKey::Paused, &paused);
}

pub fn get_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false)
}

pub fn is_paused(env: &Env) -> bool {
    get_paused(env)
}

// ── Multi-sig admin (Issue #16) ──────────────────────────────────────────────

pub fn get_admins(env: &Env) -> Vec<Address> {
    env.storage()
        .instance()
        .get::<_, Vec<Address>>(&DataKey::Admins)
        .unwrap_or(Vec::new(env))
}

pub fn set_admins(env: &Env, admins: &Vec<Address>) {
    env.storage().instance().set(&DataKey::Admins, admins);
}

/// Check whether `address` is in the multi-sig admin list.
/// Falls back to the legacy single-admin key so contracts initialised
/// before multi-sig support was added continue to work.
pub fn is_admin(env: &Env, address: &Address) -> bool {
    let admins = get_admins(env);
    if admins.len() == 0 {
        // Legacy fallback: check the single Admin key
        return env
            .storage()
            .instance()
            .get::<_, Address>(&DataKey::Admin)
            .map(|a| a == *address)
            .unwrap_or(false);
    }
    for a in admins.iter() {
        if a == *address {
            return true;
        }
    }
    false
}

pub fn get_threshold(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::Threshold)
        .unwrap_or(1)
}

pub fn set_threshold(env: &Env, threshold: u32) {
    env.storage()
        .instance()
        .set(&DataKey::Threshold, &threshold);
}

pub fn get_claim_votes(env: &Env, policy_id: u64) -> Option<ClaimVotes> {
    env.storage()
        .persistent()
        .get(&DataKey::ClaimVotes(policy_id))
}

pub fn set_claim_votes(env: &Env, policy_id: u64, votes: &ClaimVotes) {
    env.storage()
        .persistent()
        .set(&DataKey::ClaimVotes(policy_id), votes);
}

pub fn clear_claim_votes(env: &Env, policy_id: u64) {
    env.storage()
        .persistent()
        .remove(&DataKey::ClaimVotes(policy_id));
}

pub fn get_total_premium(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalPremium)
        .unwrap_or(0)
}

pub fn set_total_premium(env: &Env, amount: i128) {
    env.storage()
        .instance()
        .set(&DataKey::TotalPremium, &amount);
}

pub fn get_total_payouts(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalPayouts)
        .unwrap_or(0)
}

pub fn set_total_payouts(env: &Env, amount: i128) {
    env.storage()
        .instance()
        .set(&DataKey::TotalPayouts, &amount);
}

pub fn get_risk_pool(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::RiskPool)
}

pub fn set_risk_pool(env: &Env, risk_pool: &Address) {
    env.storage().instance().set(&DataKey::RiskPool, risk_pool);
}

pub fn get_pool_stats(env: &Env) -> PoolStats {
    let providers = get_registered_provider_vec(env);

    PoolStats {
        total_liquidity: get_total_liquidity(env),
        total_yield_distributed: get_total_yield_distributed(env),
        provider_count: providers.len(),
    }
}

pub fn set_max_policies(env: &Env, max_policies: u64) {
    env.storage()
        .instance()
        .set(&DataKey::MaxPolicies, &max_policies);
}

pub fn get_max_policies(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::MaxPolicies)
        .unwrap_or(1_000_000)
}

pub fn set_reserve_ratio(env: &Env, ratio: u32) {
    env.storage()
        .instance()
        .set(&DataKey::ReserveRatio, &ratio);
}

pub fn get_reserve_ratio(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::ReserveRatio)
        .unwrap_or(2000)
}
