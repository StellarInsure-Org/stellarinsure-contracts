use soroban_sdk::{symbol_short, Env};

use crate::{
    AdminAddedEvent, AdminRemovedEvent, ClaimProcessedEvent, ClaimSubmittedEvent,
    ClaimVoteCastEvent, ContractPausedEvent, ContractUnpausedEvent, PolicyCancelledEvent,
    PolicyCreatedEvent, PolicyExpiredEvent, PolicyRenewedEvent, PremiumPaidEvent,
    ThresholdUpdatedEvent, PayoutEvent,
};

pub fn publish_policy_created(env: &Env, event: &PolicyCreatedEvent) {
    env.events()
        .publish((symbol_short!("policy"), symbol_short!("created")), event.clone());
}

pub fn publish_premium_paid(env: &Env, event: &PremiumPaidEvent) {
    env.events()
        .publish((symbol_short!("policy"), symbol_short!("premium")), event.clone());
}

pub fn publish_claim_submitted(env: &Env, event: &ClaimSubmittedEvent) {
    env.events()
        .publish((symbol_short!("claim"), symbol_short!("submit")), event.clone());
}

pub fn publish_claim_processed(env: &Env, event: &ClaimProcessedEvent) {
    env.events()
        .publish((symbol_short!("claim"), symbol_short!("process")), event.clone());
}

pub fn publish_policy_cancelled(env: &Env, event: &PolicyCancelledEvent) {
    env.events()
        .publish((symbol_short!("policy"), symbol_short!("cancel")), event.clone());
}

pub fn publish_contract_paused(env: &Env, event: &ContractPausedEvent) {
    env.events()
        .publish((symbol_short!("contract"), symbol_short!("paused")), event.clone());
}

pub fn publish_contract_unpaused(env: &Env, event: &ContractUnpausedEvent) {
    env.events()
        .publish((symbol_short!("contract"), symbol_short!("unpaused")), event.clone());
}

// ── Issue #16 — multi-sig admin events ───────────────────────────────────────

pub fn publish_admin_added(env: &Env, event: &AdminAddedEvent) {
    env.events()
        .publish((symbol_short!("admin"), symbol_short!("added")), event.clone());
}

pub fn publish_admin_removed(env: &Env, event: &AdminRemovedEvent) {
    env.events()
        .publish((symbol_short!("admin"), symbol_short!("removed")), event.clone());
}

pub fn publish_threshold_updated(env: &Env, event: &ThresholdUpdatedEvent) {
    env.events()
        .publish((symbol_short!("admin"), symbol_short!("threshold")), event.clone());
}

pub fn publish_claim_vote_cast(env: &Env, event: &ClaimVoteCastEvent) {
    env.events()
        .publish((symbol_short!("claim"), symbol_short!("voted")), event.clone());
}

// ── Issue #22 — policy renewal event ─────────────────────────────────────────

pub fn publish_policy_renewed(env: &Env, event: &PolicyRenewedEvent) {
    env.events()
        .publish((symbol_short!("policy"), symbol_short!("renewed")), event.clone());
}

pub fn publish_policy_expired(env: &Env, event: &PolicyExpiredEvent) {
    env.events()
        .publish((symbol_short!("policy"), symbol_short!("expired")), event.clone());
}

pub fn publish_payout(env: &Env, event: &PayoutEvent) {
    env.events()
        .publish((symbol_short!("payout"), symbol_short!("transfer")), event.clone());
}
