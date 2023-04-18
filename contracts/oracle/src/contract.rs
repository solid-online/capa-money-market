use serde_json::Value;
use std::cmp::min;
use std::convert::TryFrom;
use std::str::FromStr;

use crate::error::ContractError;
use crate::state::{Config, PriceInfo, ASSETS, CONFIG};
use cosmwasm_bignumber::math::{Decimal256, Uint256};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, Addr, Attribute, Binary, Deps, DepsMut, Env, Isqrt, MessageInfo, Order,
    QueryRequest, Response, StdResult, Uint128, Uint256 as StdUint256, WasmQuery,
};

use cw_storage_plus::Bound;
use moneymarket::oracle::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, PriceResponse, PricesResponse, PricesResponseElem,
    QueryMsg, RegisterSource, Source, SourceInfoResponse, UpdateSource,
};

use astroport::asset::{AssetInfo, PairInfo};
use astroport::generator::QueryMsg as GeneratorQueryMsg;
use astroport::pair::{PoolResponse, QueryMsg as PairQueryMsg};
use astroport::querier::query_supply as cw20_query_supply;

// settings for pagination
const MAX_LIMIT: u32 = 10;
const DEFAULT_LIMIT: u32 = 10;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&msg.owner)?,
            base_asset: msg.base_asset,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterAsset { asset, source } => {
            register_asset(deps, env, info, asset, source)
        }
        ExecuteMsg::UpdateConfig { owner } => update_config(deps, info, owner),
        ExecuteMsg::UpdateSource { asset, source } => update_source(deps, env, info, asset, source),
        ExecuteMsg::FeedPrice { prices } => feed_prices(deps, env, info, prices),
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(&owner)?;
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default())
}

pub fn register_asset(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: String,
    source: RegisterSource,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if ASSETS.may_load(deps.storage, asset.clone())?.is_some() {
        return Err(ContractError::AssetAlreadyWhitelisted {});
    };

    let mut attributes: Vec<Attribute> = vec![];

    match source {
        RegisterSource::Feeder { feeder } => {
            ASSETS.save(
                deps.storage,
                asset.clone(),
                &Source::Feeder {
                    feeder,
                    price: None,
                    last_updated_time: None,
                },
            )?;

            attributes.push(attr("source_type", "feeder"));
        }
        RegisterSource::LsdContractQuery {
            contract,
            base_asset,
            query_msg,
            path_key,
            is_inverted,
        } => {
            register_lsd(
                deps,
                env,
                asset.clone(),
                base_asset,
                contract,
                query_msg,
                path_key,
                is_inverted,
            )?;

            attributes.push(attr("source_type", "lsd_contract_query"));
        }
        RegisterSource::AstroportLpAutocompound {
            vault_contract,
            generator_contract,
            pool_contract,
        } => {
            register_clp_astro(
                deps,
                env,
                asset.clone(),
                vault_contract,
                generator_contract,
                pool_contract,
            )?;

            attributes.push(attr("source_type", "astroport_lp_autocompunt"));
        }
    }

    attributes.push(attr("asset", asset));

    Ok(Response::new().add_attributes(attributes))
}

pub fn update_source(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: String,
    update_source: UpdateSource,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let mut attributes: Vec<Attribute> = vec![];

    // In case asset is not registered this return an error
    match ASSETS.load(deps.storage, asset.clone()) {
        Ok(source) => match (source, update_source) {
            (
                Source::Feeder {
                    price,
                    last_updated_time,
                    ..
                },
                UpdateSource::Feeder { feeder },
            ) => {
                ASSETS.save(
                    deps.storage,
                    asset.clone(),
                    &Source::Feeder {
                        feeder,
                        price,
                        last_updated_time,
                    },
                )?;

                attributes.push(attr("source_type", "feeder"));
            }

            (
                Source::LsdContractQuery {
                    base_asset,
                    contract,
                    query_msg,
                    path_key,
                    is_inverted,
                },
                UpdateSource::LsdContractQuery {
                    base_asset: new_base_asset,
                    contract: new_contract,
                    query_msg: new_query_msg,
                    path_key: new_path_key,
                    is_inverted: new_is_inverted,
                },
            ) => {
                register_lsd(
                    deps,
                    env,
                    asset.clone(),
                    new_base_asset.unwrap_or(base_asset),
                    new_contract.unwrap_or(contract),
                    new_query_msg.unwrap_or(query_msg),
                    new_path_key.unwrap_or(path_key),
                    new_is_inverted.unwrap_or(is_inverted),
                )?;

                attributes.push(attr("source_type", "lsd_contract_query"));
            }

            (
                Source::AstroportLpAutocompound {
                    vault_contract,
                    generator_contract,
                    pool_contract,
                    ..
                },
                UpdateSource::AstroportLpAutocompound {
                    vault_contract: new_vault_contract,
                    generator_contract: new_generator_contract,
                    pool_contract: new_pool_contract,
                },
            ) => {
                register_clp_astro(
                    deps,
                    env,
                    asset.clone(),
                    new_vault_contract.unwrap_or(vault_contract),
                    new_generator_contract.unwrap_or(generator_contract),
                    new_pool_contract.unwrap_or(pool_contract),
                )?;

                attributes.push(attr("source_type", "astroport_lp_autocompunt"));
            }

            _ => return Err(ContractError::SourceIsNotFeeder { asset }),
        },
        Err(_) => return Err(ContractError::AssetIsNotWhitelisted { asset }),
    }

    attributes.push(attr("asset", asset));

    Ok(Response::new().add_attributes(attributes))
}

pub fn feed_prices(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    prices: Vec<(String, Decimal256)>,
) -> Result<Response, ContractError> {
    let mut attributes = vec![attr("action", "feed_prices")];
    for price in prices {
        let asset: String = price.0;
        let price: Decimal256 = price.1;

        match ASSETS.load(deps.storage, asset.clone()) {
            Ok(source) => match source {
                Source::Feeder { feeder, .. } => {
                    if feeder != info.sender {
                        return Err(ContractError::Unauthorized {});
                    }
                    if price.is_zero() {
                        return Err(ContractError::NotValidZeroPrice {});
                    }

                    attributes.push(attr("asset", asset.to_string()));
                    attributes.push(attr("price", price.to_string()));

                    ASSETS.save(
                        deps.storage,
                        asset,
                        &Source::Feeder {
                            feeder,
                            price: Some(price),
                            last_updated_time: Some(env.block.time.seconds()),
                        },
                    )?
                }
                #[allow(unreachable_patterns)]
                _ => return Err(ContractError::SourceIsNotFeeder { asset }),
            },
            Err(_) => return Err(ContractError::AssetIsNotWhitelisted { asset }),
        }
    }

    Ok(Response::new().add_attributes(attributes))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps).unwrap()),
        QueryMsg::SourceInfo { asset } => to_binary(&query_source_info(deps, asset).unwrap()),
        QueryMsg::Price { base, quote } => to_binary(&query_price(deps, env, base, quote).unwrap()),
        QueryMsg::Prices { start_after, limit } => {
            to_binary(&query_prices(deps, env, start_after, limit).unwrap())
        }
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = CONFIG.load(deps.storage)?;
    let resp = ConfigResponse {
        owner: state.owner.to_string(),
        base_asset: state.base_asset,
    };

    Ok(resp)
}

fn query_source_info(deps: Deps, asset: String) -> Result<SourceInfoResponse, ContractError> {
    match ASSETS.load(deps.storage, asset.clone()) {
        Ok(source) => Ok(SourceInfoResponse { source }),
        Err(_) => Err(ContractError::AssetIsNotWhitelisted { asset }),
    }
}

fn query_price(
    deps: Deps,
    env: Env,
    base: String,
    quote: String,
) -> Result<PriceResponse, ContractError> {
    let base_price = get_price(deps, env.clone(), base)?;

    let quote_price = if CONFIG.load(deps.storage)?.base_asset == quote {
        PriceInfo {
            price: Decimal256::one(),
            last_updated_time: env.block.time.seconds(),
        }
    } else {
        get_price(deps, env, quote)?
    };

    Ok(PriceResponse {
        rate: base_price.price / quote_price.price,
        last_updated_base: base_price.last_updated_time,
        last_updated_quote: quote_price.last_updated_time,
    })
}

fn query_prices(
    deps: Deps,
    env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<PricesResponse, ContractError> {
    let curr_limit = match limit {
        Some(value) => min(value, MAX_LIMIT),
        None => DEFAULT_LIMIT,
    };

    let start: Option<Bound<String>> = start_after.map(Bound::exclusive);

    let prices: Vec<PricesResponseElem> = ASSETS
        .range(deps.storage, start, None, Order::Ascending)
        .take(usize::try_from(curr_limit).unwrap())
        .map(|item| {
            let (asset, _) = item.unwrap();

            let price_info = get_price(deps, env.clone(), asset.clone()).unwrap();

            PricesResponseElem {
                asset,
                price: price_info.price,
                last_updated_time: price_info.last_updated_time,
            }
        })
        .collect();

    Ok(PricesResponse { prices })
}

// --- FUNCTIONS ---

/// Properly register/update a LSD
#[allow(clippy::too_many_arguments)]
fn register_lsd(
    deps: DepsMut,
    env: Env,
    asset: String,
    base_asset: String,
    contract: Addr,
    query_msg: Binary,
    path_key: Vec<String>,
    is_inverted: bool,
) -> Result<(), ContractError> {
    deps.api.addr_validate(&asset)?;

    assert_asset_is_not_lsd(deps.as_ref(), asset.clone())?;

    ASSETS.save(
        deps.storage,
        asset.clone(),
        &Source::LsdContractQuery {
            base_asset,
            contract,
            query_msg,
            path_key,
            is_inverted,
        },
    )?;

    // get_price is called to check if the passed data are valids, otheriwse the tx is reverted
    get_price(deps.as_ref(), env, asset)?;

    Ok(())
}

/// Properly register/update a clp_astro
fn register_clp_astro(
    deps: DepsMut,
    env: Env,
    asset: String,
    vault_contract: Addr,
    generator_contract: Addr,
    pool_contract: Addr,
) -> Result<(), ContractError> {
    deps.api.addr_validate(&asset)?;

    let (assets, lp_contract) = pool_infos(deps.as_ref(), pool_contract.clone())?;

    ASSETS.save(
        deps.storage,
        asset.clone(),
        &Source::AstroportLpAutocompound {
            vault_contract,
            generator_contract,
            pool_contract,
            lp_contract,
            assets,
        },
    )?;

    // get_price is called to check if the passed data are valids, otheriwse the tx is reverted
    get_price(deps.as_ref(), env, asset)?;

    Ok(())
}

fn get_price(deps: Deps, env: Env, asset: String) -> Result<PriceInfo, ContractError> {
    match ASSETS.load(deps.storage, asset.clone()) {
        Ok(source) => match source {
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

            Source::LsdContractQuery {
                base_asset,
                contract,
                query_msg,
                path_key,
                is_inverted,
            } => {
                let base_asset_price_info = get_price(deps, env, base_asset)?;
                let res: Value = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: contract.to_string(),
                    msg: query_msg,
                }))?;

                let mut value = &res;

                for key in path_key {
                    value = &value[key];
                }

                Ok(PriceInfo {
                    price: {
                        if is_inverted {
                            base_asset_price_info.price
                                / Decimal256::from_str(value.as_str().unwrap())?
                        } else {
                            base_asset_price_info.price
                                * Decimal256::from_str(value.as_str().unwrap())?
                        }
                    },
                    last_updated_time: base_asset_price_info.last_updated_time,
                })
            }

            Source::AstroportLpAutocompound {
                vault_contract,
                generator_contract,
                pool_contract,
                lp_contract,
                ..
            } => astroport_lp_autocompound_price(
                deps,
                env,
                asset,
                vault_contract,
                generator_contract,
                pool_contract,
                lp_contract,
            ),
        },

        Err(_) => Err(ContractError::AssetIsNotWhitelisted { asset }),
    }
}

fn assert_asset_is_not_lsd(deps: Deps, asset: String) -> Result<(), ContractError> {
    match ASSETS.load(deps.storage, asset.clone()) {
        Ok(source) => match source {
            Source::LsdContractQuery { .. } => Err(ContractError::AssetIsLsd {}),

            _ => Ok(()),
        },

        Err(_) => Err(ContractError::AssetIsNotWhitelisted { asset }),
    }
}

/// Return:
/// - `Vec<String>`: vec of asset contract;
/// - `Addr`: Address of lp token
fn pool_infos(deps: Deps, pool_contract: Addr) -> Result<(Vec<String>, Addr), ContractError> {
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
fn pool_tokens_amount_and_price(
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

fn astroport_lp_autocompound_price(
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

fn astroport_generator_lp_deposited(
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
