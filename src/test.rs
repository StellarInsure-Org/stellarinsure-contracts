#![cfg(test)]

use crate::{PolicyType, StellarInsure, StellarInsureClient};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_create_policy() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StellarInsure);
    let client = StellarInsureClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let policyholder = Address::generate(&env);

    client.init(&admin);

    let policy_id = client.create_policy(
        &policyholder,
        &PolicyType::Weather,
        &1_000_000,
        &10_000,
        &2_592_000, // 30 days
        &String::from_str(&env, "temperature < 0"),
    );

    assert_eq!(policy_id, 0);

    let policy = client.get_policy(&policy_id);
    assert_eq!(policy.policyholder, policyholder);
    assert_eq!(policy.coverage_amount, 1_000_000);
    assert_eq!(policy.premium, 10_000);
}

#[test]
fn test_submit_and_process_claim() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StellarInsure);
    let client = StellarInsureClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let policyholder = Address::generate(&env);

    client.init(&admin);

    let policy_id = client.create_policy(
        &policyholder,
        &PolicyType::Weather,
        &1_000_000,
        &10_000,
        &2_592_000,
        &String::from_str(&env, "temperature < 0"),
    );

    // Submit claim
    client.submit_claim(
        &policy_id,
        &500_000,
        &String::from_str(&env, "Weather data proof"),
    );

    let claim = client.get_claim(&policy_id);
    assert_eq!(claim.claim_amount, 500_000);
    assert_eq!(claim.approved, false);

    // Process claim
    client.process_claim(&policy_id, &true);

    let claim = client.get_claim(&policy_id);
    assert_eq!(claim.approved, true);
}

#[test]
#[should_panic]
fn test_claim_exceeds_coverage() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StellarInsure);
    let client = StellarInsureClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let policyholder = Address::generate(&env);

    client.init(&admin);

    let policy_id = client.create_policy(
        &policyholder,
        &PolicyType::Weather,
        &1_000_000,
        &10_000,
        &2_592_000,
        &String::from_str(&env, "temperature < 0"),
    );

    // Try to claim more than coverage
    client.submit_claim(
        &policy_id,
        &2_000_000,
        &String::from_str(&env, "Weather data proof"),
    );
}
