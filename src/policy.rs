// Policy-related helper functions

use crate::{Policy, PolicyStatus};

impl Policy {
    /// Check if policy is expired
    pub fn is_expired(&self, current_time: u64) -> bool {
        current_time > self.end_time
    }

    /// Check if policy is active and valid
    pub fn is_active(&self) -> bool {
        self.status == PolicyStatus::Active
    }

    /// Check if policy can accept claims
    pub fn can_claim(&self, current_time: u64) -> bool {
        self.is_active() && !self.is_expired(current_time)
    }

    /// Calculate remaining coverage
    pub fn remaining_coverage(&self) -> i128 {
        self.coverage_amount - self.claim_amount
    }
}
