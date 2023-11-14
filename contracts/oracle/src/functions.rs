use std::{cmp::min, str::FromStr};

use astroport::generator::QueryMsg as GeneratorQueryMsg;
use astroport::{
    asset::{AssetInfo, PairInfo},
    pair::{PoolResponse, QueryMsg as PairQueryMsg},
    querier::query_supply as cw20_query_supply,
};
use cosmwasm_bignumber::math::{Decimal256, Uint256};
use cosmwasm_std::{
    to_binary, Addr, Deps, Env, Isqrt, QueryRequest, Uint128, Uint256 as StdUint256, WasmQuery,
};
use moneymarket::oracle::{PathKey, Source};
use serde_json::Value;

use crate::{
    error::ContractError,
    state::{PriceInfo, ASSETS},
};

/// Fetch the price of a specific asset
pub fn get_price(deps: Deps, env: Env, asset: String) -> Result<PriceInfo, ContractError> {
    match ASSETS.load(deps.storage, asset.clone())? {
        Source::Feeder {
            price,
            last_updated_time,
            ..
        } => {
            if let (Some(last_updated_time), Some(price)) = (last_updated_time, price) {
                Ok(PriceInfo {
                    price,
                    last_updated_time,
                })
            } else {
                Err(ContractError::PriceNeverFeeded {})
            }
        }

        Source::OnChainQuery {
            base_asset,
            query,
            path_key,
            is_inverted,
        } => {
            // let base_asset_price_info = get_price(deps, env, base_asset)?;
            let res: Value = deps.querier.query(&query)?;

            let mut value = &res;

            for key in path_key {
                match key {
                    PathKey::Index(key) => value = &value[key as usize],
                    PathKey::String(key) => value = &value[key],
                }
            }

            let mut price = Decimal256::from_str(value.as_str().unwrap())?;

            if is_inverted {
                price = Decimal256::one() / price
            }

            let last_updated_time = if let Some(base_asset) = base_asset {
                let base_asset_price_info = get_price(deps, env, base_asset)?;

                price = price * base_asset_price_info.price;

                base_asset_price_info.last_updated_time
            } else {
                env.block.time.seconds()
            };

            Ok(PriceInfo {
                price,
                last_updated_time,
            })
        }

        Source::AstroportLpVault {
            vault_contract,
            generator_contract,
            pool_contract,
            lp_contract,
            ..
        } => astroport_lp_vault_price(
            deps,
            env,
            asset,
            vault_contract,
            generator_contract,
            pool_contract,
            lp_contract,
        ),
    }
}

/// Return:
/// - `Vec<String>`: vec of asset contract;
/// - `Addr`: Address of lp token
pub fn pool_infos(deps: Deps, pool_contract: Addr) -> Result<(Vec<String>, Addr), ContractError> {
    let res: PairInfo = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pool_contract.to_string(),
        msg: to_binary(&PairQueryMsg::Pair {})?,
    }))?;

    let vec_contract_address: Vec<String> = res
        .asset_infos
        .iter()
        .map(|info| -> String {
            match info {
                AssetInfo::Token { contract_addr } => contract_addr.to_string(),
                AssetInfo::NativeToken { denom } => denom.to_string(),
            }
        })
        .collect();

    Ok((vec_contract_address, res.liquidity_token))
}

/// Return:
/// - `Vec<(String, Uint256)>`: vec of (asset contract, amount);
pub fn pool_tokens_amount_and_price(
    deps: Deps,
    env: Env,
    pool_contract: Addr,
) -> Result<Vec<(Uint256, PriceInfo)>, ContractError> {
    let res: PoolResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pool_contract.to_string(),
        msg: to_binary(&PairQueryMsg::Pool {})?,
    }))?;

    let vec_address_amount: Result<Vec<(Uint256, PriceInfo)>, ContractError> = res
        .assets
        .iter()
        .map(|asset| -> Result<(Uint256, PriceInfo), ContractError> {
            match asset.info.clone() {
                AssetInfo::Token { contract_addr } => Ok((
                    Uint256::from(asset.amount),
                    get_price(deps, env.clone(), contract_addr.to_string())?,
                )),
                AssetInfo::NativeToken { denom } => Ok((
                    Uint256::from(asset.amount),
                    get_price(deps, env.clone(), denom)?,
                )),
            }
        })
        .collect();

    vec_address_amount
}

/// The calculation of the value of liquidity token, see: https://blog.alphafinance.io/fair-lp-token-pricing/.
/// This formulation avoids a potential sandwich attack that distorts asset prices by a flashloan.
pub fn astroport_lp_vault_price(
    deps: Deps,
    env: Env,
    clp_contract: String,
    vault_contract: Addr,
    generator_contract: Addr,
    pool_contract: Addr,
    lp_contract: Addr,
) -> Result<PriceInfo, ContractError> {
    let lp_supply = Uint256::from(cw20_query_supply(&deps.querier, lp_contract.clone())?);

    let clp_supply = Uint256::from(cw20_query_supply(&deps.querier, clp_contract)?);

    let vault_lp_staked =
        astroport_generator_lp_deposited(deps, vault_contract, lp_contract, generator_contract)?;

    let vault_lp_share = Decimal256::from_ratio(vault_lp_staked, lp_supply);

    let assets_price = pool_tokens_amount_and_price(deps, env.clone(), pool_contract)?;

    // Currently we support only pools with two asset
    if assets_price.len() != 2 {
        return Err(ContractError::PoolInvalidAssetsLenght {});
    }

    let mut pool_value = Uint256::one();

    let mut last_update: u64 = env.block.time.seconds();

    for (amount, price_info) in assets_price {
        pool_value = pool_value * amount * price_info.price;

        last_update = min(last_update, price_info.last_updated_time);
    }

    // Convert pool_value from cosmwasm_bignumber::math::Decimal256 to cosmwasm_std::Uint256 in order to perform .isqrt opertation
    let pool_value = StdUint256::from(2u8) * StdUint256::from_u128(pool_value.into()).isqrt();

    let clp_price = Decimal256::from_ratio(
        vault_lp_share * Uint256::from_str(pool_value.to_string().as_str())?,
        clp_supply,
    );

    Ok(PriceInfo {
        price: clp_price,
        last_updated_time: last_update,
    })
}

/// Return the amount of a specific lp staked into generator conract for a user
pub fn astroport_generator_lp_deposited(
    deps: Deps,
    user: Addr,
    lp_contract: Addr,
    generator_contract: Addr,
) -> Result<Uint256, ContractError> {
    let res: Uint128 = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: generator_contract.to_string(),
        msg: to_binary(&GeneratorQueryMsg::Deposit {
            lp_token: lp_contract.to_string(),
            user: user.to_string(),
        })?,
    }))?;

    Ok(Uint256::from(res))
}
