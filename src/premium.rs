/// Actuarial premium calculation for StellarInsure policies.
///
/// All monetary values are in stroops (1 XLM = 10_000_000 stroops).
/// All arithmetic uses integer fixed-point math to stay `no_std`-compatible.
///
/// # Formula
///
/// ```text
/// annual_base    = coverage_amount * base_rate_bps / 10_000
/// duration_adj   = annual_base * duration_seconds / SECONDS_PER_YEAR
/// coverage_adj   = duration_adj * coverage_factor_bps / 10_000
/// premium        = coverage_adj  (minimum: MIN_PREMIUM_STROOPS)
/// ```
///
/// All divisions are integer (floor) divisions; the minimum premium floor
/// prevents underflow for very short or very small policies.
use crate::{Error, PolicyType};

/// Seconds in a 365-day year — used to annualise the duration factor.
const SECONDS_PER_YEAR: i128 = 31_536_000;

/// Minimum premium charged for any policy (0.01 XLM in stroops).
const MIN_PREMIUM_STROOPS: i128 = 100_000;

/// Precision denominator for basis-points arithmetic (1 bps = 0.01 %).
const BPS_DENOM: i128 = 10_000;

/// Coverage-tier thresholds (in stroops).
/// Tier 1: < 10 000 XLM  → no surcharge
/// Tier 2:  10 000–99 999 XLM → +50 bps
/// Tier 3:  ≥ 100 000 XLM → +100 bps
const TIER2_THRESHOLD: i128 = 100_000_000_000; // 10 000 XLM
const TIER3_THRESHOLD: i128 = 1_000_000_000_000; // 100 000 XLM

// ─── Policy-type base rates (annual, in basis points) ────────────────────────

/// Weather insurance carries higher systemic risk from correlated events.
const WEATHER_RATE_BPS: i128 = 350; // 3.50 % p.a.

/// Smart-contract (DeFi) cover is volatile but self-contained.
const SMART_CONTRACT_RATE_BPS: i128 = 300; // 3.00 % p.a.

/// Flight delay cover benefits from large, diversified pools.
const FLIGHT_RATE_BPS: i128 = 200; // 2.00 % p.a.

/// Health insurance requires large reserves; priced conservatively.
const HEALTH_RATE_BPS: i128 = 400; // 4.00 % p.a.

/// Asset protection sits between flight and weather in risk profile.
const ASSET_RATE_BPS: i128 = 250; // 2.50 % p.a.

// ─── Public API ──────────────────────────────────────────────────────────────

/// Calculate the premium for a new policy.
///
/// # Arguments
/// * `policy_type`      – determines the actuarial base rate
/// * `coverage_amount`  – maximum payout in stroops (must be > 0)
/// * `duration_seconds` – policy lifetime in seconds (must be > 0)
///
/// # Returns
/// The premium in stroops, always ≥ `MIN_PREMIUM_STROOPS`.
///
/// # Errors
/// * `Error::InvalidAmount`   – `coverage_amount` is ≤ 0
/// * `Error::InvalidDuration` – `duration_seconds` is 0
pub fn calculate_premium(
    policy_type: &PolicyType,
    coverage_amount: i128,
    duration_seconds: u64,
) -> Result<i128, Error> {
    if coverage_amount <= 0 {
        return Err(Error::InvalidAmount);
    }
    if duration_seconds == 0 {
        return Err(Error::InvalidDuration);
    }

    let base_rate = base_rate_bps(policy_type);
    let duration = duration_seconds as i128;

    // Step 1 — annualised base premium
    // annual_base = coverage_amount * base_rate / 10_000
    let annual_base = coverage_amount
        .checked_mul(base_rate)
        .and_then(|v| v.checked_div(BPS_DENOM))
        .ok_or(Error::InvalidAmount)?;

    // Step 2 — pro-rate for actual duration
    // duration_adj = annual_base * duration / SECONDS_PER_YEAR
    let duration_adj = annual_base
        .checked_mul(duration)
        .and_then(|v| v.checked_div(SECONDS_PER_YEAR))
        .ok_or(Error::InvalidAmount)?;

    // Step 3 — coverage-concentration surcharge
    let coverage_factor = coverage_factor_bps(coverage_amount);
    // coverage_adj = duration_adj * (10_000 + surcharge_bps) / 10_000
    let premium = duration_adj
        .checked_mul(BPS_DENOM + coverage_factor)
        .and_then(|v| v.checked_div(BPS_DENOM))
        .ok_or(Error::InvalidAmount)?;

    // Step 4 — enforce minimum floor
    Ok(premium.max(MIN_PREMIUM_STROOPS))
}

// ─── Private helpers ─────────────────────────────────────────────────────────

/// Annual base rate in basis points, differentiated by policy type.
fn base_rate_bps(policy_type: &PolicyType) -> i128 {
    match policy_type {
        PolicyType::Weather => WEATHER_RATE_BPS,
        PolicyType::SmartContract => SMART_CONTRACT_RATE_BPS,
        PolicyType::Flight => FLIGHT_RATE_BPS,
        PolicyType::Health => HEALTH_RATE_BPS,
        PolicyType::Asset => ASSET_RATE_BPS,
    }
}

/// Additional basis points added to account for concentration risk when a
/// policy's coverage amount falls in a higher tier.
fn coverage_factor_bps(coverage_amount: i128) -> i128 {
    if coverage_amount >= TIER3_THRESHOLD {
        100 // +1.00 % surcharge for very large policies
    } else if coverage_amount >= TIER2_THRESHOLD {
        50 // +0.50 % surcharge for medium policies
    } else {
        0 // no surcharge for small policies
    }
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const ONE_XLM: i128 = 10_000_000; // stroops

    /// 30-day duration in seconds.
    const THIRTY_DAYS: u64 = 30 * 24 * 3600;

    /// 365-day duration in seconds.
    const ONE_YEAR: u64 = 365 * 24 * 3600;

    // ── Error cases ──────────────────────────────────────────────────────────

    #[test]
    fn rejects_zero_coverage() {
        let err = calculate_premium(&PolicyType::Weather, 0, THIRTY_DAYS).unwrap_err();
        assert_eq!(err, Error::InvalidAmount);
    }

    #[test]
    fn rejects_negative_coverage() {
        let err = calculate_premium(&PolicyType::Flight, -1, THIRTY_DAYS).unwrap_err();
        assert_eq!(err, Error::InvalidAmount);
    }

    #[test]
    fn rejects_zero_duration() {
        let err = calculate_premium(&PolicyType::Health, ONE_XLM * 1000, 0).unwrap_err();
        assert_eq!(err, Error::InvalidDuration);
    }

    // ── Minimum floor ────────────────────────────────────────────────────────

    #[test]
    fn applies_minimum_premium_for_tiny_policy() {
        // 1 XLM coverage for 1 second → raw premium is essentially 0
        let premium = calculate_premium(&PolicyType::Flight, ONE_XLM, 1).unwrap();
        assert_eq!(premium, MIN_PREMIUM_STROOPS);
    }

    // ── Rate ordering ────────────────────────────────────────────────────────

    #[test]
    fn health_more_expensive_than_flight() {
        let coverage = ONE_XLM * 1_000;
        let health = calculate_premium(&PolicyType::Health, coverage, ONE_YEAR).unwrap();
        let flight = calculate_premium(&PolicyType::Flight, coverage, ONE_YEAR).unwrap();
        assert!(health > flight, "health={health} flight={flight}");
    }

    #[test]
    fn weather_more_expensive_than_asset() {
        let coverage = ONE_XLM * 1_000;
        let weather = calculate_premium(&PolicyType::Weather, coverage, ONE_YEAR).unwrap();
        let asset = calculate_premium(&PolicyType::Asset, coverage, ONE_YEAR).unwrap();
        assert!(weather > asset, "weather={weather} asset={asset}");
    }

    #[test]
    fn all_policy_types_produce_positive_premiums() {
        let coverage = ONE_XLM * 5_000;
        let types = [
            PolicyType::Weather,
            PolicyType::SmartContract,
            PolicyType::Flight,
            PolicyType::Health,
            PolicyType::Asset,
        ];
        for pt in &types {
            let p = calculate_premium(pt, coverage, ONE_YEAR).unwrap();
            assert!(p > 0, "{pt:?} produced non-positive premium");
        }
    }

    // ── Duration scaling ─────────────────────────────────────────────────────

    #[test]
    fn longer_duration_yields_higher_premium() {
        let coverage = ONE_XLM * 10_000;
        let short = calculate_premium(&PolicyType::Asset, coverage, THIRTY_DAYS).unwrap();
        let long = calculate_premium(&PolicyType::Asset, coverage, ONE_YEAR).unwrap();
        assert!(long > short, "long={long} short={short}");
    }

    #[test]
    fn one_year_premium_roughly_matches_annual_rate() {
        // 1 000 XLM coverage, Asset (2.50 % p.a.), tier-1 (no surcharge)
        // expected ≈ 1_000 * 10_000_000 * 250 / 10_000 = 25_000_000 stroops = 2.5 XLM
        let coverage = ONE_XLM * 1_000;
        let premium = calculate_premium(&PolicyType::Asset, coverage, ONE_YEAR).unwrap();
        // Allow ±1 stroop for integer rounding
        let expected = ONE_XLM * 25; // 2.5 XLM
        let delta = (premium - expected).abs();
        assert!(
            delta <= 1,
            "premium={premium} expected≈{expected} delta={delta}"
        );
    }

    // ── Coverage tier surcharge ───────────────────────────────────────────────

    #[test]
    fn tier2_coverage_more_expensive_per_unit_than_tier1() {
        let tier1 = ONE_XLM * 5_000; // < 10 000 XLM
        let tier2 = ONE_XLM * 50_000; // ≥ 10 000 XLM, < 100 000 XLM

        // Normalise to per-XLM cost to compare concentration-adjusted rates
        let p1 = calculate_premium(&PolicyType::Flight, tier1, ONE_YEAR).unwrap();
        let p2 = calculate_premium(&PolicyType::Flight, tier2, ONE_YEAR).unwrap();
        let rate1 = p1 * BPS_DENOM / tier1;
        let rate2 = p2 * BPS_DENOM / tier2;
        assert!(rate2 > rate1, "rate2={rate2} rate1={rate1}");
    }

    #[test]
    fn tier3_coverage_more_expensive_per_unit_than_tier2() {
        let tier2 = ONE_XLM * 50_000;
        let tier3 = ONE_XLM * 500_000;

        let p2 = calculate_premium(&PolicyType::Flight, tier2, ONE_YEAR).unwrap();
        let p3 = calculate_premium(&PolicyType::Flight, tier3, ONE_YEAR).unwrap();
        let rate2 = p2 * BPS_DENOM / tier2;
        let rate3 = p3 * BPS_DENOM / tier3;
        assert!(rate3 > rate2, "rate3={rate3} rate2={rate2}");
    }

    // ── Determinism ──────────────────────────────────────────────────────────

    #[test]
    fn calculation_is_deterministic() {
        let coverage = ONE_XLM * 8_000;
        let duration = THIRTY_DAYS * 3;
        let p1 = calculate_premium(&PolicyType::SmartContract, coverage, duration).unwrap();
        let p2 = calculate_premium(&PolicyType::SmartContract, coverage, duration).unwrap();
        assert_eq!(p1, p2);
    }
}
