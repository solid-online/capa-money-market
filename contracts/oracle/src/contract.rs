use serde_json::Value;
use std::cmp::{min};
use std::convert::TryFrom;
use std::str::FromStr;

use crate::error::ContractError;
use crate::state::{Config, PriceInfo, ASSETS, CONFIG};
use cosmwasm_bignumber::math::Decimal256;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, Addr, Attribute, Binary, Deps, DepsMut, Env, MessageInfo, Order, QueryRequest,
    Response, StdResult, WasmQuery,
};
use cw_storage_plus::Bound;
use moneymarket::oracle::{
    ConfigResponse, ExecuteMsg, FeederResponse, InstantiateMsg, PriceResponse, PriceSource,
    PricesResponse, PricesResponseElem, QueryMsg,
};

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
        ExecuteMsg::RegisterAsset { asset, source } => register_asset(deps, info, asset, source),
        ExecuteMsg::UpdateConfig { owner } => update_config(deps, info, owner),
        ExecuteMsg::UpdateFeeder { asset, feeder } => update_feeder(deps, info, asset, feeder),
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
    info: MessageInfo,
    asset: String,
    source: PriceSource,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if ASSETS.may_load(deps.storage, asset.clone())?.is_some() {
        return Err(ContractError::AssetAlreadyWhitelisted {});
    };

    let mut attributes: Vec<Attribute> = vec![];

    match source.clone() {
        PriceSource::Feeder { feeder, .. } => {
            ASSETS.save(
                deps.storage,
                asset,
                &PriceSource::Feeder {
                    feeder: feeder.clone(),
                    price: None,
                    last_updated_time: None,
                },
            )?;

            attributes.push(attr("source_type", "feeder"));
            attributes.push(attr("feeder", feeder));
        }
        PriceSource::LsdContractQuery { contract, base_asset, .. } => {

            assert_asset_is_not_lsd(deps.as_ref(), base_asset)?;

            ASSETS.save(deps.storage, asset, &source)?;

            attributes.push(attr("source_type", "lsd_contract_query"));
            attributes.push(attr("contract", contract));
        }
    }

    Ok(Response::new().add_attributes(attributes))
}

pub fn update_feeder(
    deps: DepsMut,
    info: MessageInfo,
    asset: String,
    new_feeder: Addr,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // In case asset is not registered this return an error
    match ASSETS.load(deps.storage, asset.clone()) {
        Ok(source) => {
            match source {
                PriceSource::Feeder {
                    price,
                    last_updated_time,
                    ..
                } => {
                    // Is better to use .save instead to use .update because i've alredt matched the type of PriceSource
                    ASSETS.save(
                        deps.storage,
                        asset.clone(),
                        &PriceSource::Feeder {
                            feeder: new_feeder.clone(),
                            price,
                            last_updated_time,
                        },
                    )?
                }
                #[allow(unreachable_patterns)]
                _ => return Err(ContractError::SourceIsNotFeeder { asset }),
            }
        }
        Err(_) => return Err(ContractError::AssetIsNotWhitelisted {}),
    }

    Ok(Response::new().add_attributes(vec![
        attr("action", "update_feeder"),
        attr("asset", asset),
        attr("feeder", new_feeder),
    ]))
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
                PriceSource::Feeder { feeder, .. } => {
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
                        &PriceSource::Feeder {
                            feeder,
                            price: Some(price),
                            last_updated_time: Some(env.block.time.seconds()),
                        },
                    )?
                }
                #[allow(unreachable_patterns)]
                _ => return Err(ContractError::SourceIsNotFeeder { asset }),
            },
            Err(_) => return Err(ContractError::AssetIsNotWhitelisted {}),
        }
    }

    Ok(Response::new().add_attributes(attributes))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps).unwrap()),
        QueryMsg::Feeder { asset } => to_binary(&query_feeder(deps, asset).unwrap()),
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

fn query_feeder(deps: Deps, asset: String) -> Result<FeederResponse, ContractError> {
    match ASSETS.load(deps.storage, asset.clone())? {
        PriceSource::Feeder { feeder, .. } => Ok(FeederResponse {
            asset,
            feeder: feeder.to_string(),
        }),
        #[allow(unreachable_patterns)]
        _ => Err(ContractError::SourceIsNotFeeder { asset }),
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

fn get_price(deps: Deps, env: Env, asset: String) -> Result<PriceInfo, ContractError> {
    match ASSETS.load(deps.storage, asset) {
        Ok(source) => match source {
            PriceSource::Feeder {
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

            PriceSource::LsdContractQuery {
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

                for  key in path_key {
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
        },

        Err(_) => Err(ContractError::AssetIsNotWhitelisted {}),
    }
}

fn assert_asset_is_not_lsd (deps: Deps, asset:String) -> Result<(), ContractError> {

    match ASSETS.load(deps.storage, asset) {
        Ok(source) => match source {

            PriceSource::LsdContractQuery {..} => {
                return Err(ContractError::AssetIsLsd {})

            }

            _ => return Ok(())
        },

        Err(_) => Err(ContractError::AssetIsNotWhitelisted {}),
    }
}