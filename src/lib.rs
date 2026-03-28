#![no_std]

mod error;
mod events;
mod multisig;
mod policy;
mod premium;
mod risk_pool;
mod storage;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, token::TokenClient, Address, Env, String, Vec};

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
        // Bootstrap multi-sig admin list with the initial admin (threshold = 1)
        let mut admins = Vec::new(&env);
        admins.push_back(admin.clone());
        storage::set_admins(&env, &admins);
        storage::set_threshold(&env, 1);
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
        if storage::is_paused(&env) {
            return Err(Error::ContractPaused);
        }
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
        if storage::is_paused(&env) {
            return Err(Error::ContractPaused);
        }
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
        if storage::is_paused(&env) {
            return Err(Error::ContractPaused);
        }
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
        if storage::is_paused(&env) {
            return Err(Error::ContractPaused);
        }
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

    /// Calculate the premium for a prospective policy without creating it.
    ///
    /// This is a read-only helper so callers can present accurate pricing
    /// in a UI before the policyholder commits to paying gas.
    ///
    /// # Arguments
    /// * `policy_type`      – insurance category that drives the actuarial rate
    /// * `coverage_amount`  – desired maximum payout in stroops (> 0)
    /// * `duration_seconds` – intended policy lifetime in seconds (> 0)
    ///
    /// # Returns
    /// The required premium in stroops.
    pub fn calculate_premium(
        _env: Env,
        policy_type: PolicyType,
        coverage_amount: i128,
        duration_seconds: u64,
    ) -> Result<i128, Error> {
        premium::calculate_premium(&policy_type, coverage_amount, duration_seconds)
    }

    /// Get policy details
    pub fn get_policy(env: Env, policy_id: u64) -> Result<Policy, Error> {
        storage::get_policy(&env, policy_id)
    }

    /// Get claim details
    pub fn get_claim(env: Env, policy_id: u64) -> Result<Claim, Error> {
        storage::get_claim(&env, policy_id)
    }

    /// Pause the contract — emergency circuit breaker (admin only).
    /// Blocks `create_policy`, `pay_premium`, `submit_claim`, and `cancel_policy`.
    pub fn pause(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        let current_admin = storage::get_admin(&env);
        if admin != current_admin {
            return Err(Error::Unauthorized);
        }
        if storage::is_paused(&env) {
            return Ok(()); // idempotent
        }
        storage::set_paused(&env, true);
        events::publish_contract_paused(
            &env,
            &ContractPausedEvent {
                admin,
                timestamp: env.ledger().timestamp(),
            },
        );
        Ok(())
    }

    /// Unpause the contract (admin only).
    pub fn unpause(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        let current_admin = storage::get_admin(&env);
        if admin != current_admin {
            return Err(Error::Unauthorized);
        }
        if !storage::is_paused(&env) {
            return Ok(()); // idempotent
        }
        storage::set_paused(&env, false);
        events::publish_contract_unpaused(
            &env,
            &ContractUnpausedEvent {
                admin,
                timestamp: env.ledger().timestamp(),
            },
        );
        Ok(())
    }

    /// Query whether the contract is currently paused.
    pub fn get_paused(env: Env) -> bool {
        storage::is_paused(&env)
    }

    // ── Issue #16 — Multi-sig admin functions ─────────────────────────────────

    /// Add a new admin to the multi-sig admin list (any existing admin can call).
    pub fn add_admin(env: Env, caller: Address, new_admin: Address) -> Result<(), Error> {
        multisig::require_admin(&env, &caller)?;
        if storage::is_admin(&env, &new_admin) {
            return Err(Error::AdminAlreadyExists);
        }
        let mut admins = storage::get_admins(&env);
        admins.push_back(new_admin.clone());
        storage::set_admins(&env, &admins);
        events::publish_admin_added(
            &env,
            &AdminAddedEvent { caller, new_admin },
        );
        Ok(())
    }

    /// Remove an admin from the multi-sig admin list.
    /// The last remaining admin cannot be removed.
    /// Threshold is automatically lowered if it would exceed admin count.
    pub fn remove_admin(env: Env, caller: Address, target: Address) -> Result<(), Error> {
        multisig::require_admin(&env, &caller)?;
        if !storage::is_admin(&env, &target) {
            return Err(Error::AdminNotFound);
        }
        let admins = storage::get_admins(&env);
        if admins.len() <= 1 {
            return Err(Error::Unauthorized); // cannot remove the last admin
        }
        let mut filtered = Vec::new(&env);
        for a in admins.iter() {
            if a != target {
                filtered.push_back(a);
            }
        }
        // Ensure threshold never exceeds remaining admin count
        let threshold = storage::get_threshold(&env);
        if threshold > filtered.len() {
            storage::set_threshold(&env, filtered.len());
        }
        storage::set_admins(&env, &filtered);
        events::publish_admin_removed(
            &env,
            &AdminRemovedEvent { caller, removed_admin: target },
        );
        Ok(())
    }

    /// Update the approval threshold for multi-sig claim voting.
    /// Must be between 1 and the current number of admins (inclusive).
    pub fn set_threshold(env: Env, caller: Address, threshold: u32) -> Result<(), Error> {
        multisig::require_admin(&env, &caller)?;
        if threshold == 0 {
            return Err(Error::InvalidThreshold);
        }
        let admins = storage::get_admins(&env);
        if threshold > admins.len() {
            return Err(Error::InvalidThreshold);
        }
        storage::set_threshold(&env, threshold);
        events::publish_threshold_updated(
            &env,
            &ThresholdUpdatedEvent { caller, new_threshold: threshold },
        );
        Ok(())
    }

    /// Cast an approval or rejection vote on a pending claim.
    /// When the approval threshold is reached the claim is automatically
    /// approved and the payout transferred. When rejection becomes
    /// mathematically forced the claim is automatically rejected.
    pub fn vote_claim(
        env: Env,
        policy_id: u64,
        voter: Address,
        approve: bool,
    ) -> Result<(), Error> {
        if storage::is_paused(&env) {
            return Err(Error::ContractPaused);
        }
        multisig::require_admin(&env, &voter)?;

        let mut policy = storage::get_policy(&env, policy_id)?;
        if policy.status != PolicyStatus::ClaimPending {
            return Err(Error::NoPendingClaim);
        }

        let mut votes = storage::get_claim_votes(&env, policy_id).unwrap_or(ClaimVotes {
            approvals: Vec::new(&env),
            rejections: Vec::new(&env),
        });

        // Prevent double voting
        if multisig::vec_contains(&votes.approvals, &voter)
            || multisig::vec_contains(&votes.rejections, &voter)
        {
            return Err(Error::AlreadyVoted);
        }

        if approve {
            votes.approvals.push_back(voter.clone());
        } else {
            votes.rejections.push_back(voter.clone());
        }

        let approval_count = votes.approvals.len();
        let rejection_count = votes.rejections.len();
        storage::set_claim_votes(&env, policy_id, &votes);

        events::publish_claim_vote_cast(
            &env,
            &ClaimVoteCastEvent {
                policy_id,
                voter,
                approve,
                approval_count,
                rejection_count,
            },
        );

        // Auto-finalise when threshold is reached
        if multisig::threshold_reached(&env, approval_count) {
            let mut claim = storage::get_claim(&env, policy_id)?;
            policy.status = PolicyStatus::ClaimApproved;
            claim.approved = true;
            let token_address =
                storage::get_premium_token(&env).ok_or(Error::NotInitialized)?;
            let token_client = TokenClient::new(&env, &token_address);
            token_client.transfer(
                &env.current_contract_address(),
                &policy.policyholder,
                &claim.claim_amount,
            );
            storage::set_policy(&env, policy_id, &policy);
            storage::set_claim(&env, policy_id, &claim);
            storage::clear_claim_votes(&env, policy_id);
            events::publish_claim_processed(
                &env,
                &ClaimProcessedEvent {
                    policy_id,
                    policyholder: policy.policyholder,
                    claim_amount: claim.claim_amount,
                    approved: true,
                    status: PolicyStatus::ClaimApproved,
                },
            );
        } else if multisig::rejection_forced(&env, rejection_count) {
            let claim = storage::get_claim(&env, policy_id)?;
            policy.status = PolicyStatus::ClaimRejected;
            policy.claim_amount = 0;
            storage::set_policy(&env, policy_id, &policy);
            storage::clear_claim_votes(&env, policy_id);
            events::publish_claim_processed(
                &env,
                &ClaimProcessedEvent {
                    policy_id,
                    policyholder: policy.policyholder,
                    claim_amount: claim.claim_amount,
                    approved: false,
                    status: PolicyStatus::ClaimRejected,
                },
            );
        }

        Ok(())
    }

    /// Return the current list of admins.
    pub fn get_admins(env: Env) -> Vec<Address> {
        storage::get_admins(&env)
    }

    /// Return the current approval threshold.
    pub fn get_threshold(env: Env) -> u32 {
        storage::get_threshold(&env)
    }

    // ── Issue #22 — Policy renewal ────────────────────────────────────────────

    /// Grace period after expiry during which a policyholder may still renew
    /// (7 days expressed in ledger seconds).
    const RENEWAL_GRACE_PERIOD: u64 = 604_800;

    /// Renew a policy for an additional `duration` seconds.
    ///
    /// Allowed when the policy is `Active` and either not yet expired or
    /// expired within the 7-day grace period. The renewal premium (same as
    /// the original premium) is collected immediately via token transfer.
    pub fn renew_policy(env: Env, policy_id: u64, duration: u64) -> Result<(), Error> {
        if storage::is_paused(&env) {
            return Err(Error::ContractPaused);
        }

        let mut policy = storage::get_policy(&env, policy_id)?;
        policy.policyholder.require_auth();

        if duration == 0 {
            return Err(Error::InvalidDuration);
        }

        // Only Active policies can be renewed
        if policy.status != PolicyStatus::Active {
            return Err(Error::PolicyNotRenewable);
        }

        let current_time = env.ledger().timestamp();

        // If the policy has already expired, enforce the grace window
        if current_time > policy.end_time {
            if current_time > policy.end_time + Self::RENEWAL_GRACE_PERIOD {
                return Err(Error::RenewalGracePeriodExpired);
            }
        }

        // New end time extends from the later of now or the current end
        let base = if current_time > policy.end_time {
            current_time
        } else {
            policy.end_time
        };
        let new_end_time = base + duration;

        // Collect renewal premium
        let token_address =
            storage::get_premium_token(&env).ok_or(Error::NotInitialized)?;
        let token_client = TokenClient::new(&env, &token_address);
        token_client.transfer(
            &policy.policyholder,
            &env.current_contract_address(),
            &policy.premium,
        );

        let renewal_premium = policy.premium;
        policy.end_time = new_end_time;
        policy.status = PolicyStatus::Active; // reset if it was effectively expired

        storage::set_policy(&env, policy_id, &policy);

        events::publish_policy_renewed(
            &env,
            &PolicyRenewedEvent {
                policy_id,
                policyholder: policy.policyholder,
                new_end_time,
                renewal_premium,
            },
        );

        Ok(())
    }

    /// Get current contract version
    pub fn version(env: Env) -> u32 {
        storage::get_version(&env)
    }

    /// Upgrade the contract code (admin only)
    pub fn upgrade(env: Env, new_wasm_hash: soroban_sdk::BytesN<32>) -> Result<(), Error> {
        let admin = storage::get_admin(&env);
        admin.require_auth();

        let current_version = storage::get_version(&env);
        storage::set_version(&env, current_version + 1);

        env.deployer().update_current_contract_wasm(new_wasm_hash);
        
        // Emit an upgrade event if we have one defined, but here we'll just return successfully.
        Ok(())
    }
}
