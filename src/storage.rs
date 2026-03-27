use soroban_sdk::{contracttype, Address, Env, Vec};

use crate::{Claim, Error, Policy, PoolStats, ProviderPosition, Providers};

#[contracttype]
enum DataKey {
    Admin,
    PolicyCounter,
    Policy(u64),
    Claim(u64),
    RiskPoolAdmin,
    TotalLiquidity,
    TotalYieldDistributed,
    Provider(Address),
    Providers,
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
    env.storage().instance().set(&DataKey::TotalLiquidity, &amount);
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

pub fn get_pool_stats(env: &Env) -> PoolStats {
    let providers = get_registered_provider_vec(env);

    PoolStats {
        total_liquidity: get_total_liquidity(env),
        total_yield_distributed: get_total_yield_distributed(env),
        provider_count: providers.len(),
    }
}
