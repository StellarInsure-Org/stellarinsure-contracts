use soroban_sdk::{contracttype, Address, Env, String, Symbol};

/// Oracle interface trait for external data feeds
/// Supports price, weather, flight, and other parametric data sources
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum OracleError {
    NotSupported,
    VerificationFailed,
    DataUnavailable,
    OracleNotRegistered,
}

/// Result returned from oracle verification
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct OracleResult {
    pub is_verified: bool,
    pub details: String,
}

/// Oracle provider trait for implementing custom data feeds
pub trait OracleProvider {
    fn verify_condition(env: &Env, parameter: Symbol) -> Result<OracleResult, OracleError>;
}

/// Weather oracle for parametric weather insurance
/// Verifies conditions like temperature, rainfall, wind speed
pub struct WeatherOracle;
impl OracleProvider for WeatherOracle {
    fn verify_condition(env: &Env, _parameter: Symbol) -> Result<OracleResult, OracleError> {
        // Production implementation would query external weather data feed
        // Example: Check if rainfall > threshold, temperature < freezing, etc.
        Ok(OracleResult {
            is_verified: true,
            details: String::from_str(env, "Weather conditions verified safely"),
        })
    }
}

/// Flight oracle for flight delay/cancellation insurance
/// Verifies flight status from aviation data providers
pub struct FlightOracle;
impl OracleProvider for FlightOracle {
    fn verify_condition(env: &Env, _parameter: Symbol) -> Result<OracleResult, OracleError> {
        // Production implementation would query flight status APIs
        // Example: Check if flight delayed > 2 hours, cancelled, etc.
        Ok(OracleResult {
            is_verified: true,
            details: String::from_str(env, "Flight condition verified"),
        })
    }
}

/// Smart contract oracle for on-chain event verification
/// Monitors blockchain state and contract events
pub struct SmartContractOracle;
impl OracleProvider for SmartContractOracle {
    fn verify_condition(env: &Env, _parameter: Symbol) -> Result<OracleResult, OracleError> {
        // Production implementation would verify on-chain conditions
        // Example: Check contract state, token balances, transaction events
        Ok(OracleResult {
            is_verified: true,
            details: String::from_str(env, "Telemetry confirms valid state"),
        })
    }
}

/// Price oracle for asset price-based insurance
/// Tracks cryptocurrency and asset prices
pub struct PriceOracle;
impl OracleProvider for PriceOracle {
    fn verify_condition(env: &Env, _parameter: Symbol) -> Result<OracleResult, OracleError> {
        // Production implementation would query price feeds
        // Example: Check if asset price dropped below threshold
        Ok(OracleResult {
            is_verified: true,
            details: String::from_str(env, "Price condition verified"),
        })
    }
}

/// Oracle registry entry storing oracle address and metadata
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct OracleRegistry {
    pub oracle_address: Address,
    pub oracle_type: Symbol,
    pub is_active: bool,
}
