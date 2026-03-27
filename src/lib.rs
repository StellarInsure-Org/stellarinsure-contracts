#![no_std]

mod error;
mod events;
mod policy;
mod risk_pool;
mod storage;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, token::TokenClient, Address, Env, String};

pub use error::Error;
pub use risk_pool::{RiskPool, RiskPoolClient};
pub use types::*;

#[contract]
pub struct StellarInsure;

#[contractimpl]
impl StellarInsure {
    /// Initialize the insurance protocol
    pub fn init(env: Env, admin: Address) {
        storage::set_admin(&env, &admin);
        storage::set_policy_counter(&env, 0);
    }

    /// Set the token used for premiums and payouts (admin only)
    pub fn set_premium_token(env: Env, admin: Address, token: Address) {
        admin.require_auth();
        let current_admin = storage::get_admin(&env);
        if admin != current_admin {
            panic!("Unauthorized");
        }
        storage::set_premium_token(&env, &token);
    }

    /// Create a new insurance policy
    ///
    /// # Arguments
    /// * `policyholder` - Address of the policy holder
    /// * `policy_type` - Type of insurance (Weather, SmartContract, Flight, etc.)
    /// * `coverage_amount` - Maximum payout amount
    /// * `premium` - Premium amount to be paid
    /// * `duration` - Policy duration in seconds
    /// * `trigger_condition` - Encoded trigger condition
    pub fn create_policy(
        env: Env,
        policyholder: Address,
        policy_type: PolicyType,
        coverage_amount: i128,
        premium: i128,
        duration: u64,
        trigger_condition: String,
    ) -> Result<u64, Error> {
        policyholder.require_auth();

        if coverage_amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        if premium <= 0 {
            return Err(Error::InvalidPremium);
        }

        if duration == 0 {
            return Err(Error::InvalidDuration);
        }

        let policy_id = storage::get_policy_counter(&env);
        let next_id = policy_id + 1;

        let policy = Policy {
            id: policy_id,
            policyholder: policyholder.clone(),
            policy_type,
            coverage_amount,
            premium,
            start_time: env.ledger().timestamp(),
            end_time: env.ledger().timestamp() + duration,
            trigger_condition,
            status: PolicyStatus::Active,
            claim_amount: 0,
        };

        storage::set_policy(&env, policy_id, &policy);
        storage::set_policy_counter(&env, next_id);
        events::publish_policy_created(
            &env,
            &PolicyCreatedEvent {
                policy_id,
                policyholder,
                policy_type: policy.policy_type.clone(),
                coverage_amount: policy.coverage_amount,
                premium: policy.premium,
                start_time: policy.start_time,
                end_time: policy.end_time,
                trigger_condition: policy.trigger_condition.clone(),
            },
        );

        Ok(policy_id)
    }

    /// Pay premium for a policy
    pub fn pay_premium(env: Env, policy_id: u64, amount: i128) -> Result<(), Error> {
        let policy = storage::get_policy(&env, policy_id)?;

        if policy.status != PolicyStatus::Active {
            return Err(Error::PolicyNotActive);
        }

        policy.policyholder.require_auth();

        if amount != policy.premium {
            return Err(Error::InvalidPremium);
        }

        // [SECURITY] Implement actual token transfer for premium (#14)
        // In this implementation, we transfer from policyholder to the contract.
        // The contract address can be obtained via env.current_contract_address().
        let token_address = storage::get_premium_token(&env).ok_or(Error::NotInitialized)?;
        let token_client = TokenClient::new(&env, &token_address);
        token_client.transfer(&policy.policyholder, &env.current_contract_address(), &amount);

        events::publish_premium_paid(
            &env,
            &PremiumPaidEvent {
                policy_id,
                policyholder: policy.policyholder,
                amount,
            },
        );

        Ok(())
    }

    /// Submit a claim for payout
    pub fn submit_claim(
        env: Env,
        policy_id: u64,
        claim_amount: i128,
        proof: String,
    ) -> Result<(), Error> {
        let mut policy = storage::get_policy(&env, policy_id)?;

        if policy.status != PolicyStatus::Active {
            return Err(Error::PolicyNotActive);
        }

        policy.policyholder.require_auth();

        if claim_amount > policy.coverage_amount {
            return Err(Error::ClaimExceedsCoverage);
        }

        if claim_amount <= 0 {
            return Err(Error::InvalidClaimAmount);
        }

        if env.ledger().timestamp() > policy.end_time {
            return Err(Error::PolicyExpired);
        }

        policy.claim_amount = claim_amount;
        policy.status = PolicyStatus::ClaimPending;

        storage::set_policy(&env, policy_id, &policy);
        storage::set_claim(
            &env,
            policy_id,
            &Claim {
                policy_id,
                claim_amount,
                proof: proof.clone(),
                timestamp: env.ledger().timestamp(),
                approved: false,
            },
        );
        events::publish_claim_submitted(
            &env,
            &ClaimSubmittedEvent {
                policy_id,
                policyholder: policy.policyholder.clone(),
                claim_amount,
                proof,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Approve or reject a claim (admin only)
    pub fn process_claim(env: Env, policy_id: u64, approved: bool) -> Result<(), Error> {
        let admin = storage::get_admin(&env);
        admin.require_auth();

        let mut policy = storage::get_policy(&env, policy_id)?;
        let mut claim = storage::get_claim(&env, policy_id)?;

        if policy.status != PolicyStatus::ClaimPending {
            return Err(Error::NoPendingClaim);
        }

        if approved {
            policy.status = PolicyStatus::ClaimApproved;
            claim.approved = true;

            // [SECURITY] Implement actual token transfer for payout (#14)
            let token_address = storage::get_premium_token(&env).ok_or(Error::NotInitialized)?;
            let token_client = TokenClient::new(&env, &token_address);
            token_client.transfer(&env.current_contract_address(), &policy.policyholder, &claim.claim_amount);
        } else {
            policy.status = PolicyStatus::ClaimRejected;
            policy.claim_amount = 0;
        }

        storage::set_policy(&env, policy_id, &policy);
        storage::set_claim(&env, policy_id, &claim);
        events::publish_claim_processed(
            &env,
            &ClaimProcessedEvent {
                policy_id,
                policyholder: policy.policyholder,
                claim_amount: claim.claim_amount,
                approved,
                status: policy.status,
            },
        );

        Ok(())
    }

    /// Cancel a policy
    pub fn cancel_policy(env: Env, policy_id: u64) -> Result<(), Error> {
        let mut policy = storage::get_policy(&env, policy_id)?;

        policy.policyholder.require_auth();

        if policy.status != PolicyStatus::Active {
            return Err(Error::PolicyNotActive);
        }

        policy.status = PolicyStatus::Cancelled;
        storage::set_policy(&env, policy_id, &policy);
        events::publish_policy_cancelled(
            &env,
            &PolicyCancelledEvent {
                policy_id,
                policyholder: policy.policyholder,
            },
        );

        Ok(())
    }

    /// Get policy details
    pub fn get_policy(env: Env, policy_id: u64) -> Result<Policy, Error> {
        storage::get_policy(&env, policy_id)
    }

    /// Get claim details
    pub fn get_claim(env: Env, policy_id: u64) -> Result<Claim, Error> {
        storage::get_claim(&env, policy_id)
    }
}
