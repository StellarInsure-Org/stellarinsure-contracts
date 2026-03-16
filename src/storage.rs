use soroban_sdk::{Address, Env, String};

use crate::{Claim, Error, Policy};

const ADMIN_KEY: &str = "ADMIN";
const POLICY_COUNTER: &str = "POLICY_CTR";

fn policy_key(policy_id: u64) -> String {
    String::from_str(&Env::default(), &format!("POLICY_{}", policy_id))
}

fn claim_key(policy_id: u64) -> String {
    String::from_str(&Env::default(), &format!("CLAIM_{}", policy_id))
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage()
        .instance()
        .set(&String::from_str(env, ADMIN_KEY), admin);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&String::from_str(env, ADMIN_KEY))
        .unwrap()
}

pub fn set_policy_counter(env: &Env, counter: u64) {
    env.storage()
        .instance()
        .set(&String::from_str(env, POLICY_COUNTER), &counter);
}

pub fn get_policy_counter(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&String::from_str(env, POLICY_COUNTER))
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
