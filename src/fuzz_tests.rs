#![cfg(test)]

use crate::{
    PolicyStatus, PolicyType, RiskPool, RiskPoolClient, StellarInsure, StellarInsureClient,
};
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, Env, String};

const ONE_XLM: i128 = 10_000_000;

fn next_u64(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn rand_u64(state: &mut u64, min: u64, max: u64) -> u64 {
    debug_assert!(min <= max);
    min + (next_u64(state) % (max - min + 1))
}

fn rand_i128(state: &mut u64, min: i128, max: i128) -> i128 {
    debug_assert!(min <= max);
    let span = (max - min + 1) as u128;
    min + (next_u64(state) as u128 % span) as i128
}

fn random_policy_type(state: &mut u64) -> PolicyType {
    match next_u64(state) % 5 {
        0 => PolicyType::Weather,
        1 => PolicyType::SmartContract,
        2 => PolicyType::Flight,
        3 => PolicyType::Health,
        _ => PolicyType::Asset,
    }
}

fn setup_insurance_contract() -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StellarInsure);
    let client = StellarInsureClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let policyholder = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_address = env.register_stellar_asset_contract(token_admin);

    let sac = StellarAssetClient::new(&env, &token_address);
    sac.mint(&policyholder, &(ONE_XLM * 50_000_000));
    sac.mint(&contract_id, &(ONE_XLM * 50_000_000));

    client.init(&admin);
    client.set_premium_token(&admin, &token_address);

    (env, contract_id, admin, policyholder)
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
fn fuzz_premium_calculation_random_inputs() {
    let (env, contract_id, _admin, _policyholder) = setup_insurance_contract();
    let client = StellarInsureClient::new(&env, &contract_id);
    let mut seed = 0xD00DFEED_12345678;

    for _ in 0..400 {
        let policy_type = random_policy_type(&mut seed);
        let coverage = rand_i128(&mut seed, ONE_XLM, ONE_XLM * 1_000_000);
        let duration = rand_u64(&mut seed, 1, 365 * 24 * 3600 * 5);

        let premium = client.calculate_premium(&policy_type, &coverage, &duration);
        assert!(premium > 0, "premium must always be positive");

        let doubled = coverage.saturating_mul(2);
        let premium_doubled = client.calculate_premium(&policy_type, &doubled, &duration);
        assert!(
            premium_doubled >= premium,
            "premium should be monotonic for larger coverage"
        );
    }
}

#[test]
fn fuzz_policy_creation_edge_cases_random_inputs() {
    let mut seed = 0xA11C_C0DE_5566_7788;

    for _ in 0..80 {
        let (env, contract_id, _admin, policyholder) = setup_insurance_contract();
        let client = StellarInsureClient::new(&env, &contract_id);
        let policy_type = random_policy_type(&mut seed);
        let coverage = rand_i128(&mut seed, ONE_XLM, ONE_XLM * 2_000_000);
        let duration = rand_u64(&mut seed, 1, 365 * 24 * 3600 * 3);
        let premium = client.calculate_premium(&policy_type, &coverage, &duration);

        let created_id = client.create_policy(
            &policyholder,
            &policy_type,
            &coverage,
            &premium,
            &duration,
            &String::from_str(&env, "fuzz-policy"),
        );
        let stored = client.get_policy(&created_id);

        assert_eq!(created_id, 0);
        assert_eq!(stored.policyholder, policyholder);
        assert_eq!(stored.coverage_amount, coverage);
        assert_eq!(stored.premium, premium);
        assert_eq!(stored.status, PolicyStatus::Active);
    }
}

#[test]
fn fuzz_claim_submission_boundary_values() {
    let mut seed = 0xFACEB00C_0BADF00D;

    for _ in 0..80 {
        let (env, contract_id, _admin, policyholder) = setup_insurance_contract();
        let client = StellarInsureClient::new(&env, &contract_id);
        let coverage = rand_i128(&mut seed, ONE_XLM, ONE_XLM * 500_000);
        let duration = rand_u64(&mut seed, 60, 365 * 24 * 3600);
        let premium = client.calculate_premium(&PolicyType::Weather, &coverage, &duration);

        let policy_id = client.create_policy(
            &policyholder,
            &PolicyType::Weather,
            &coverage,
            &premium,
            &duration,
            &String::from_str(&env, "fuzz-claim"),
        );

        let claim_amount = rand_i128(&mut seed, 1, coverage);
        client.submit_claim(&policy_id, &claim_amount, &String::from_str(&env, "proof"));
        let claim = client.get_claim(&policy_id);
        assert_eq!(claim.claim_amount, claim_amount);
        assert!(!claim.approved);
    }
}

#[test]
fn fuzz_risk_pool_arithmetic_randomized() {
    let mut seed = 0x1234_5678_9ABC_DEF0;

    for _ in 0..80 {
        let (_env, contract_id, _admin, provider_one, provider_two) = setup_risk_pool();
        let client = RiskPoolClient::new(&_env, &contract_id);
        let c1 = rand_i128(&mut seed, 1_000, 1_000_000_000);
        let c2 = rand_i128(&mut seed, 1_000, 1_000_000_000);
        let yield_amount = rand_i128(&mut seed, 1, 1_000_000_000);

        client.add_liquidity(&provider_one, &c1);
        client.add_liquidity(&provider_two, &c2);
        let stats_before = client.get_pool_stats();
        assert_eq!(stats_before.total_liquidity, c1 + c2);

        client.distribute_yield(&yield_amount);
        let p1 = client.get_provider_position(&provider_one);
        let p2 = client.get_provider_position(&provider_two);
        assert!(p1.accrued_yield >= 0);
        assert!(p2.accrued_yield >= 0);
        assert!(
            p1.accrued_yield + p2.accrued_yield <= yield_amount,
            "distribution should not over-allocate yield"
        );

        let withdraw_one = rand_i128(&mut seed, 1, c1);
        let withdraw_two = rand_i128(&mut seed, 1, c2);
        client.withdraw_liquidity(&provider_one, &withdraw_one);
        client.withdraw_liquidity(&provider_two, &withdraw_two);

        let p1_after = client.get_provider_position(&provider_one);
        let p2_after = client.get_provider_position(&provider_two);
        assert_eq!(p1_after.contribution, c1 - withdraw_one);
        assert_eq!(p2_after.contribution, c2 - withdraw_two);
    }
}

#[test]
fn fuzz_risk_pool_overflow_underflow_edges_are_caught() {
    let (env, contract_id, _admin, provider_one, _provider_two) = setup_risk_pool();
    let client = RiskPoolClient::new(&env, &contract_id);

    client.add_liquidity(&provider_one, &i128::MAX);
    let overflow = client.try_add_liquidity(&provider_one, &1);
    assert!(overflow.is_err(), "overflow edge must fail");

    client.withdraw_liquidity(&provider_one, &(i128::MAX - 1));
    let underflow = client.try_withdraw_liquidity(&provider_one, &2);
    assert!(underflow.is_err(), "underflow edge must fail");
}

#[test]
fn fuzz_claim_submission_above_coverage_rejected() {
    let mut seed = 0x0DDC0FFE_77AA55CC;

    for _ in 0..70 {
        let (env, contract_id, _admin, policyholder) = setup_insurance_contract();
        let client = StellarInsureClient::new(&env, &contract_id);
        let coverage = rand_i128(&mut seed, ONE_XLM, ONE_XLM * 200_000);
        let duration = rand_u64(&mut seed, 60, 365 * 24 * 3600);
        let premium = client.calculate_premium(&PolicyType::Asset, &coverage, &duration);
        let policy_id = client.create_policy(
            &policyholder,
            &PolicyType::Asset,
            &coverage,
            &premium,
            &duration,
            &String::from_str(&env, "fuzz-over-coverage"),
        );

        let too_large = coverage + 1;
        let result =
            client.try_submit_claim(&policy_id, &too_large, &String::from_str(&env, "proof"));
        assert!(result.is_err(), "claim above coverage must fail");
    }
}
