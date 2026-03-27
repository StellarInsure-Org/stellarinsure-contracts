use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

use crate::{
    storage, Error, LiquidityAddedEvent, LiquidityWithdrawnEvent, PoolStats, ProviderPosition,
    YieldClaimedEvent, YieldDistributionEvent,
};

#[contract]
pub struct RiskPool;

#[contractimpl]
impl RiskPool {
    pub fn init(env: Env, admin: Address) -> Result<(), Error> {
        if storage::has_risk_pool_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }

        storage::set_risk_pool_admin(&env, &admin);
        storage::set_total_liquidity(&env, 0);
        storage::set_total_yield_distributed(&env, 0);

        Ok(())
    }

    pub fn add_liquidity(env: Env, provider: Address, amount: i128) -> Result<(), Error> {
        provider.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let mut position = storage::get_provider(&env, &provider).unwrap_or(ProviderPosition {
            provider: provider.clone(),
            contribution: 0,
            accrued_yield: 0,
        });

        position.contribution += amount;

        let new_total = storage::get_total_liquidity(&env) + amount;
        storage::set_provider(&env, &provider, &position);
        storage::ensure_provider_registered(&env, &provider);
        storage::set_total_liquidity(&env, new_total);

        env.events().publish(
            (symbol_short!("pool"), symbol_short!("deposit")),
            LiquidityAddedEvent {
                provider,
                amount,
                new_contribution: position.contribution,
                pool_balance: new_total,
            },
        );

        Ok(())
    }

    pub fn withdraw_liquidity(env: Env, provider: Address, amount: i128) -> Result<(), Error> {
        provider.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let mut position = storage::get_provider(&env, &provider).ok_or(Error::ProviderNotFound)?;

        if amount > position.contribution {
            return Err(Error::InsufficientLiquidity);
        }

        position.contribution -= amount;
        let new_total = storage::get_total_liquidity(&env) - amount;
        storage::set_total_liquidity(&env, new_total);

        if position.contribution == 0 && position.accrued_yield == 0 {
            storage::remove_provider(&env, &provider);
            storage::unregister_provider(&env, &provider);
        } else {
            storage::set_provider(&env, &provider, &position);
        }

        env.events().publish(
            (symbol_short!("pool"), symbol_short!("withdraw")),
            LiquidityWithdrawnEvent {
                provider,
                amount,
                remaining_contribution: position.contribution,
                pool_balance: new_total,
            },
        );

        Ok(())
    }

    pub fn distribute_yield(env: Env, amount: i128) -> Result<(), Error> {
        let admin = storage::get_risk_pool_admin(&env);
        admin.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let total_liquidity = storage::get_total_liquidity(&env);
        if total_liquidity <= 0 {
            return Err(Error::InsufficientLiquidity);
        }

        let providers = storage::get_registered_provider_vec(&env);
        for provider in providers.iter() {
            let mut position = storage::get_provider(&env, &provider).ok_or(Error::ProviderNotFound)?;
            let share = amount * position.contribution / total_liquidity;
            position.accrued_yield += share;
            storage::set_provider(&env, &provider, &position);
        }

        storage::set_total_yield_distributed(
            &env,
            storage::get_total_yield_distributed(&env) + amount,
        );

        env.events().publish(
            (symbol_short!("pool"), symbol_short!("yield")),
            YieldDistributionEvent {
                amount,
                total_liquidity_before: total_liquidity,
            },
        );

        Ok(())
    }

    pub fn claim_yield(env: Env, provider: Address) -> Result<i128, Error> {
        provider.require_auth();

        let mut position = storage::get_provider(&env, &provider).ok_or(Error::ProviderNotFound)?;
        if position.accrued_yield <= 0 {
            return Err(Error::NoYieldAvailable);
        }

        let amount = position.accrued_yield;
        position.accrued_yield = 0;
        storage::set_provider(&env, &provider, &position);

        env.events().publish(
            (symbol_short!("pool"), symbol_short!("claim")),
            YieldClaimedEvent { provider, amount },
        );

        Ok(amount)
    }

    pub fn get_provider_position(env: Env, provider: Address) -> Result<ProviderPosition, Error> {
        storage::get_provider(&env, &provider).ok_or(Error::ProviderNotFound)
    }

    pub fn get_pool_balance(env: Env) -> i128 {
        storage::get_total_liquidity(&env)
    }

    pub fn get_pool_stats(env: Env) -> PoolStats {
        storage::get_pool_stats(&env)
    }
}
