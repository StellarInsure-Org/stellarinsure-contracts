use soroban_sdk::{contracttype, Address, Env};

use crate::{Claim, Error, Policy};

#[contracttype]
enum DataKey {
    Admin,
    PolicyCounter,
    Policy(u64),
    Claim(u64),
}

fn policy_key(policy_id: u64) -> DataKey {
    DataKey::Policy(policy_id)
}

fn claim_key(policy_id: u64) -> DataKey {
    DataKey::Claim(policy_id)
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
