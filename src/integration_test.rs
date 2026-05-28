#![cfg(test)]

//! Cross-contract integration tests for StellarInsure and RiskPool
//!
//! Tests verify interactions between the main contract, risk pool, and oracle contracts.
//! Covers premium-to-risk-pool flow, claim-to-payout flow, admin functions, and error handling.

use crate::{
    PolicyStatus, PolicyType, RiskPool, RiskPoolClient, StellarInsure, StellarInsureClient,
};
use soroban_sdk::{
    testutils::Address as _,
    token::StellarAssetClient,
    Address, Env, String,
};

/// Set up both main contract and risk pool with token
fn setup_integrated_contracts<'a>() -> (
    Env,
    Address,
    Address,
    Address,
    Address,
    StellarInsureClient<'a>,
    RiskPoolClient<'a>,
) {
    let env = Env::default();
    env.mock_all_auths();

    // Register main contract
    let main_contract_id = env.register_contract(None, StellarInsure);
    let main_client = StellarInsureClient::new(&env, &main_contract_id);

    // Register risk pool contract
    let risk_pool_id = env.register_contract(None, RiskPool);
    let risk_pool_client = RiskPoolClient::new(&env, &risk_pool_id);

    // Create addresses
    let admin = Address::generate(&env);
    let policyholder = Address::generate(&env);
    let provider = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_address = env.register_stellar_asset_contract_v2(token_admin).address();

    // Mint tokens
    let sac = StellarAssetClient::new(&env, &token_address);
    sac.mint(&policyholder, &10_000_000);
    sac.mint(&provider, &10_000_000);
    sac.mint(&main_contract_id, &5_000_000);
    sac.mint(&risk_pool_id, &5_000_000);

    // Initialize contracts
    main_client.init(&admin);
    main_client.set_premium_token(&admin, &token_address);
    main_client.set_risk_pool(&admin, &risk_pool_id);

    risk_pool_client.init(&admin);

    (
        env,
        main_contract_id,
        risk_pool_id,
        admin,
        policyholder,
        main_client,
        risk_pool_client,
    )
}

#[test]
fn test_policy_creation_with_risk_pool_deposit() {
    let (env, main_id, pool_id, _admin, policyholder, main_client, pool_client) =
        setup_integrated_contracts();

    // Provider adds liquidity to risk pool
    pool_client.add_liquidity(&policyholder, &1_000_000);
    let pool_balance_before = pool_client.get_pool_balance();
    assert_eq!(pool_balance_before, 1_000_000);

    // Create policy
    let policy_id = main_client.create_policy(
        &policyholder,
        &PolicyType::Weather,
        &500_000,
        &10_000,
        &2_592_000,
        &String::from_str(&env, "rainfall < 50mm"),
    );

    // Pay premium - should flow to risk pool
    main_client.pay_premium(&policy_id, &10_000);

    // Verify risk pool received premium
    let pool_balance_after = pool_client.get_pool_balance();
    assert_eq!(pool_balance_after, 1_010_000);

    // Verify policy is active
    let policy = main_client.get_policy(&policy_id);
    assert_eq!(policy.status, PolicyStatus::Active);
}

#[test]
fn test_claim_payout_with_risk_pool_withdrawal() {
    let (env, main_id, pool_id, admin, policyholder, main_client, pool_client) =
        setup_integrated_contracts();

    // Provider adds liquidity
    pool_client.add_liquidity(&policyholder, &2_000_000);

    // Create and pay for policy
    let policy_id = main_client.create_policy(
        &policyholder,
        &PolicyType::Weather,
        &500_000,
        &10_000,
        &2_592_000,
        &String::from_str(&env, "rainfall < 50mm"),
    );
    main_client.pay_premium(&policy_id, &10_000);

    let pool_balance_before = pool_client.get_pool_balance();

    // Submit claim
    main_client.submit_claim(&policy_id, &500_000, &String::from_str(&env, "evidence"));

    // Process claim (admin approves)
    main_client.process_claim(&policy_id, &true);

    // Verify risk pool balance decreased by payout amount
    let pool_balance_after = pool_client.get_pool_balance();
    assert_eq!(pool_balance_after, pool_balance_before - 500_000);

    // Verify policy status
    let policy = main_client.get_policy(&policy_id);
    assert_eq!(policy.status, PolicyStatus::ClaimApproved);
}

#[test]
fn test_admin_functions_across_contracts() {
    let (env, main_id, pool_id, admin, policyholder, main_client, pool_client) =
        setup_integrated_contracts();

    // Test main contract admin functions
    main_client.set_max_policies(&admin, &500_000);
    assert_eq!(main_client.get_max_policies(), 500_000);

    // Test risk pool admin functions
    pool_client.set_reserve_ratio(&admin, &3000); // 30%
    assert_eq!(pool_client.get_reserve_ratio(), 3000);

    // Test pause/unpause
    main_client.pause(&admin);
    assert_eq!(main_client.get_paused(), true);

    main_client.unpause(&admin);
    assert_eq!(main_client.get_paused(), false);
}

#[test]
#[should_panic(expected = "InsufficientLiquidity")]
fn test_claim_payout_fails_when_pool_insufficient() {
    let (env, main_id, pool_id, admin, policyholder, main_client, pool_client) =
        setup_integrated_contracts();

    // Add minimal liquidity
    pool_client.add_liquidity(&policyholder, &100_000);

    // Create policy with large coverage
    let policy_id = main_client.create_policy(
        &policyholder,
        &PolicyType::Weather,
        &500_000,
        &10_000,
        &2_592_000,
        &String::from_str(&env, "rainfall < 50mm"),
    );
    main_client.pay_premium(&policy_id, &10_000);

    // Submit and approve claim for amount exceeding pool balance
    main_client.submit_claim(&policy_id, &500_000, &String::from_str(&env, "evidence"));
    main_client.process_claim(&policy_id, &true); // Should panic
}

#[test]
#[should_panic(expected = "NotInitialized")]
fn test_pay_premium_fails_when_risk_pool_not_set() {
    let env = Env::default();
    env.mock_all_auths();

    let main_contract_id = env.register_contract(None, StellarInsure);
    let main_client = StellarInsureClient::new(&env, &main_contract_id);

    let admin = Address::generate(&env);
    let policyholder = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_address = env.register_stellar_asset_contract_v2(token_admin).address();

    let sac = StellarAssetClient::new(&env, &token_address);
    sac.mint(&policyholder, &10_000_000);

    main_client.init(&admin);
    main_client.set_premium_token(&admin, &token_address);
    // Note: NOT setting risk pool

    let policy_id = main_client.create_policy(
        &policyholder,
        &PolicyType::Weather,
        &500_000,
        &10_000,
        &2_592_000,
        &String::from_str(&env, "rainfall < 50mm"),
    );

    // This should work without risk pool
    main_client.pay_premium(&policy_id, &10_000);
}

#[test]
fn test_multiple_policies_with_shared_risk_pool() {
    let (env, main_id, pool_id, admin, policyholder, main_client, pool_client) =
        setup_integrated_contracts();

    let policyholder2 = Address::generate(&env);
    let sac = StellarAssetClient::new(&env, &env.register_stellar_asset_contract_v2(admin.clone()).address());
    sac.mint(&policyholder2, &10_000_000);

    // Add liquidity
    pool_client.add_liquidity(&policyholder, &3_000_000);

    // Create multiple policies
    let policy_id_1 = main_client.create_policy(
        &policyholder,
        &PolicyType::Weather,
        &500_000,
        &10_000,
        &2_592_000,
        &String::from_str(&env, "rainfall < 50mm"),
    );

    let policy_id_2 = main_client.create_policy(
        &policyholder2,
        &PolicyType::Flight,
        &300_000,
        &6_000,
        &1_296_000,
        &String::from_str(&env, "delay > 120min"),
    );

    // Pay premiums
    main_client.pay_premium(&policy_id_1, &10_000);
    main_client.pay_premium(&policy_id_2, &6_000);

    // Verify pool received both premiums
    let pool_balance = pool_client.get_pool_balance();
    assert_eq!(pool_balance, 3_016_000);

    // Submit and process first claim
    main_client.submit_claim(&policy_id_1, &500_000, &String::from_str(&env, "evidence1"));
    main_client.process_claim(&policy_id_1, &true);

    // Verify pool balance after first payout
    let pool_balance_after_1 = pool_client.get_pool_balance();
    assert_eq!(pool_balance_after_1, 2_516_000);

    // Submit and process second claim
    main_client.submit_claim(&policy_id_2, &300_000, &String::from_str(&env, "evidence2"));
    main_client.process_claim(&policy_id_2, &true);

    // Verify pool balance after second payout
    let pool_balance_after_2 = pool_client.get_pool_balance();
    assert_eq!(pool_balance_after_2, 2_216_000);
}

#[test]
fn test_risk_pool_reserve_ratio_blocks_withdrawal() {
    let (env, main_id, pool_id, admin, policyholder, main_client, pool_client) =
        setup_integrated_contracts();

    // Add liquidity
    pool_client.add_liquidity(&policyholder, &1_000_000);

    // Set reserve ratio to 50%
    pool_client.set_reserve_ratio(&admin, &5000);

    // Try to withdraw more than available (should respect reserve)
    // Available = 1_000_000 - (1_000_000 * 0.5) = 500_000
    let result = pool_client.try_withdraw_liquidity(&policyholder, &600_000);
    assert!(result.is_err());

    // Withdraw within available amount should succeed
    pool_client.withdraw_liquidity(&policyholder, &400_000);
    let balance = pool_client.get_pool_balance();
    assert_eq!(balance, 600_000);
}

#[test]
fn test_yield_distribution_across_providers() {
    let (env, main_id, pool_id, admin, policyholder, main_client, pool_client) =
        setup_integrated_contracts();

    let provider2 = Address::generate(&env);
    let sac = StellarAssetClient::new(&env, &env.register_stellar_asset_contract_v2(admin.clone()).address());
    sac.mint(&provider2, &10_000_000);

    // Two providers add liquidity
    pool_client.add_liquidity(&policyholder, &600_000);
    pool_client.add_liquidity(&provider2, &400_000);

    // Distribute yield
    pool_client.distribute_yield(&100_000);

    // Check provider positions
    let position1 = pool_client.get_provider_position(&policyholder);
    let position2 = pool_client.get_provider_position(&provider2);

    // Provider 1 should get 60% of yield (60,000)
    assert_eq!(position1.accrued_yield, 60_000);
    // Provider 2 should get 40% of yield (40,000)
    assert_eq!(position2.accrued_yield, 40_000);
}

#[test]
fn test_contract_pause_blocks_operations() {
    let (env, main_id, pool_id, admin, policyholder, main_client, pool_client) =
        setup_integrated_contracts();

    // Pause contract
    main_client.pause(&admin);

    // Try to create policy - should fail
    let result = main_client.try_create_policy(
        &policyholder,
        &PolicyType::Weather,
        &500_000,
        &10_000,
        &2_592_000,
        &String::from_str(&env, "rainfall < 50mm"),
    );
    assert!(result.is_err());

    // Unpause
    main_client.unpause(&admin);

    // Now should succeed
    let policy_id = main_client.create_policy(
        &policyholder,
        &PolicyType::Weather,
        &500_000,
        &10_000,
        &2_592_000,
        &String::from_str(&env, "rainfall < 50mm"),
    );
    assert_eq!(policy_id, 0);
}
