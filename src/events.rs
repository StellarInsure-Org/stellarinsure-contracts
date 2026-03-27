use soroban_sdk::{symbol_short, Env};

use crate::{
    ClaimProcessedEvent, ClaimSubmittedEvent, PolicyCancelledEvent, PolicyCreatedEvent,
    PremiumPaidEvent,
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
