//! Multi-signature helper utilities for admin-gated operations.
//!
//! All admin functions that touch configuration or claim payouts must go
//! through these helpers so that security invariants are enforced uniformly.

use soroban_sdk::{Address, Env};

use crate::{storage, Error};

// ---------------------------------------------------------------------------
// Auth helpers
// ---------------------------------------------------------------------------

/// Require that `caller` is in the admin list AND has provided a valid
/// Soroban authorisation for this invocation.
///
/// Returns `Error::Unauthorized` if the address is not an admin.
pub fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
    caller.require_auth();
    if !storage::is_admin(env, caller) {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Threshold helpers
// ---------------------------------------------------------------------------

/// Returns `true` when `count` approvals satisfy the stored threshold.
pub fn threshold_reached(env: &Env, count: u32) -> bool {
    count >= storage::get_threshold(env)
}

/// Returns `true` when enough reject votes have been cast that the approve
/// threshold can *never* be reached.
///
/// Condition: `reject_count > total_admins - threshold`
pub fn rejection_forced(env: &Env, reject_count: u32) -> bool {
    let total = storage::get_admins(env).len();
    let threshold = storage::get_threshold(env);
    // If threshold > total (should not happen but be safe), force reject.
    if threshold > total {
        return true;
    }
    reject_count > total - threshold
}

// ---------------------------------------------------------------------------
// Vec-of-Address helpers (no std available in no_std Soroban env)
// ---------------------------------------------------------------------------

/// Returns `true` if `addr` is present in `list`.
pub fn vec_contains(list: &soroban_sdk::Vec<Address>, addr: &Address) -> bool {
    for item in list.iter() {
        if item == *addr {
            return true;
        }
    }
    false
}
