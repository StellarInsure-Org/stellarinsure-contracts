#![cfg(test)]

use crate::{
    PolicyStatus, PolicyType, RiskPool, RiskPoolClient, StellarInsure, StellarInsureClient,
};
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    Address, Env, String,
};

fn setup_insurance_contract() -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StellarInsure);
    let client = StellarInsureClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let policyholder = Address::generate(&env);
    client.init(&admin);

    (env, contract_id, admin, policyholder)
}

fn create_policy(
    env: &Env,
    client: &StellarInsureClient,
    policyholder: &Address,
) -> u64 {
    client.create_policy(
        policyholder,
        &PolicyType::Weather,
        &1_000_000,
        &10_000,
        &2_592_000,
        &String::from_str(env, "temperature < 0"),
    )
}

fn setup_risk_pool() -> (Env, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RiskPool);
    let client = RiskPoolClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let provider_one = Address::generate(&env);
    let provider_two = Address::generate(&env);
    client.init(&admin);

    (env, contract_id, admin, provider_one, provider_two)
}

#[test]
fn test_create_policy_stores_expected_values() {
    let (env, contract_id, _admin, policyholder) = setup_insurance_contract();
    let client = StellarInsureClient::new(&env, &contract_id);

    let policy_id = create_policy(&env, &client, &policyholder);
    let policy = client.get_policy(&policy_id);

    assert_eq!(policy_id, 0);
    assert_eq!(policy.policyholder, policyholder);
    assert_eq!(policy.coverage_amount, 1_000_000);
    assert_eq!(policy.premium, 10_000);
    assert_eq!(policy.status, PolicyStatus::Active);
}

#[test]
fn test_create_policy_emits_event() {
    let (env, contract_id, _admin, policyholder) = setup_insurance_contract();
    let client = StellarInsureClient::new(&env, &contract_id);

    create_policy(&env, &client, &policyholder);

    assert_eq!(env.events().all().len(), 1);
}

#[test]
#[should_panic]
fn test_create_policy_rejects_zero_duration() {
    let (env, contract_id, _admin, policyholder) = setup_insurance_contract();
    let client = StellarInsureClient::new(&env, &contract_id);

    client.create_policy(
        &policyholder,
        &PolicyType::Weather,
        &1_000_000,
        &10_000,
        &0,
        &String::from_str(&env, "temperature < 0"),
    );
}

#[test]
fn test_pay_premium_emits_event() {
    let (env, contract_id, _admin, policyholder) = setup_insurance_contract();
    let client = StellarInsureClient::new(&env, &contract_id);

    let policy_id = create_policy(&env, &client, &policyholder);
    client.pay_premium(&policy_id, &10_000);

    assert_eq!(env.events().all().len(), 2);
}

#[test]
fn test_submit_claim_sets_pending_status_and_emits_event() {
    let (env, contract_id, _admin, policyholder) = setup_insurance_contract();
    let client = StellarInsureClient::new(&env, &contract_id);

    let policy_id = create_policy(&env, &client, &policyholder);
    client.submit_claim(
        &policy_id,
        &500_000,
        &String::from_str(&env, "Weather data proof"),
    );

    let policy = client.get_policy(&policy_id);
    let claim = client.get_claim(&policy_id);
    assert_eq!(policy.status, PolicyStatus::ClaimPending);
    assert_eq!(claim.claim_amount, 500_000);
    assert!(!claim.approved);
    assert_eq!(env.events().all().len(), 2);
}

#[test]
#[should_panic]
fn test_submit_claim_rejects_zero_amount() {
    let (env, contract_id, _admin, policyholder) = setup_insurance_contract();
    let client = StellarInsureClient::new(&env, &contract_id);

    let policy_id = create_policy(&env, &client, &policyholder);
    client.submit_claim(&policy_id, &0, &String::from_str(&env, "proof"));
}

#[test]
#[should_panic]
fn test_submit_claim_rejects_amount_over_coverage() {
    let (env, contract_id, _admin, policyholder) = setup_insurance_contract();
    let client = StellarInsureClient::new(&env, &contract_id);

    let policy_id = create_policy(&env, &client, &policyholder);
    client.submit_claim(&policy_id, &2_000_000, &String::from_str(&env, "proof"));
}

#[test]
#[should_panic]
fn test_submit_claim_rejects_expired_policy() {
    let (env, contract_id, _admin, policyholder) = setup_insurance_contract();
    let client = StellarInsureClient::new(&env, &contract_id);

    let policy_id = create_policy(&env, &client, &policyholder);
    env.ledger().with_mut(|ledger| {
        ledger.timestamp += 2_592_001;
    });

    client.submit_claim(&policy_id, &500_000, &String::from_str(&env, "proof"));
}

#[test]
fn test_process_claim_approve_updates_claim_and_policy() {
    let (env, contract_id, _admin, policyholder) = setup_insurance_contract();
    let client = StellarInsureClient::new(&env, &contract_id);

    let policy_id = create_policy(&env, &client, &policyholder);
    client.submit_claim(&policy_id, &500_000, &String::from_str(&env, "proof"));
    client.process_claim(&policy_id, &true);

    let policy = client.get_policy(&policy_id);
    let claim = client.get_claim(&policy_id);
    assert_eq!(policy.status, PolicyStatus::ClaimApproved);
    assert!(claim.approved);
    assert_eq!(env.events().all().len(), 3);
}

#[test]
fn test_process_claim_reject_sets_rejected_status() {
    let (env, contract_id, _admin, policyholder) = setup_insurance_contract();
    let client = StellarInsureClient::new(&env, &contract_id);

    let policy_id = create_policy(&env, &client, &policyholder);
    client.submit_claim(&policy_id, &500_000, &String::from_str(&env, "proof"));
    client.process_claim(&policy_id, &false);

    let policy = client.get_policy(&policy_id);
    let claim = client.get_claim(&policy_id);
    assert_eq!(policy.status, PolicyStatus::ClaimRejected);
    assert!(!claim.approved);
    assert_eq!(policy.claim_amount, 0);
}

#[test]
#[should_panic]
fn test_process_claim_requires_pending_claim() {
    let (env, contract_id, _admin, policyholder) = setup_insurance_contract();
    let client = StellarInsureClient::new(&env, &contract_id);

    let policy_id = create_policy(&env, &client, &policyholder);
    client.process_claim(&policy_id, &true);
}

#[test]
fn test_cancel_policy_updates_status_and_emits_event() {
    let (env, contract_id, _admin, policyholder) = setup_insurance_contract();
    let client = StellarInsureClient::new(&env, &contract_id);

    let policy_id = create_policy(&env, &client, &policyholder);
    client.cancel_policy(&policy_id);

    let policy = client.get_policy(&policy_id);
    assert_eq!(policy.status, PolicyStatus::Cancelled);
    assert_eq!(env.events().all().len(), 2);
}

#[test]
fn test_risk_pool_tracks_contributions_and_stats() {
    let (env, contract_id, _admin, provider_one, provider_two) = setup_risk_pool();
    let client = RiskPoolClient::new(&env, &contract_id);

    client.add_liquidity(&provider_one, &1_000);
    client.add_liquidity(&provider_two, &3_000);

    let stats = client.get_pool_stats();
    let position_one = client.get_provider_position(&provider_one);
    let position_two = client.get_provider_position(&provider_two);

    assert_eq!(stats.total_liquidity, 4_000);
    assert_eq!(stats.provider_count, 2);
    assert_eq!(position_one.contribution, 1_000);
    assert_eq!(position_two.contribution, 3_000);
}

#[test]
fn test_risk_pool_distributes_yield_fairly() {
    let (env, contract_id, _admin, provider_one, provider_two) = setup_risk_pool();
    let client = RiskPoolClient::new(&env, &contract_id);

    client.add_liquidity(&provider_one, &1_000);
    client.add_liquidity(&provider_two, &3_000);
    client.distribute_yield(&400);

    let position_one = client.get_provider_position(&provider_one);
    let position_two = client.get_provider_position(&provider_two);
    let stats = client.get_pool_stats();

    assert_eq!(position_one.accrued_yield, 100);
    assert_eq!(position_two.accrued_yield, 300);
    assert_eq!(stats.total_yield_distributed, 400);
}

#[test]
fn test_risk_pool_claim_yield_and_withdraw() {
    let (env, contract_id, _admin, provider_one, _provider_two) = setup_risk_pool();
    let client = RiskPoolClient::new(&env, &contract_id);

    client.add_liquidity(&provider_one, &1_000);
    client.distribute_yield(&100);

    let claimed = client.claim_yield(&provider_one);
    client.withdraw_liquidity(&provider_one, &500);
    let position = client.get_provider_position(&provider_one);

    assert_eq!(claimed, 100);
    assert_eq!(position.contribution, 500);
    assert_eq!(position.accrued_yield, 0);
    assert_eq!(client.get_pool_balance(), 500);
}

#[test]
#[should_panic]
fn test_risk_pool_rejects_over_withdrawal() {
    let (env, contract_id, _admin, provider_one, _provider_two) = setup_risk_pool();
    let client = RiskPoolClient::new(&env, &contract_id);

    client.add_liquidity(&provider_one, &100);
    client.withdraw_liquidity(&provider_one, &200);
}
