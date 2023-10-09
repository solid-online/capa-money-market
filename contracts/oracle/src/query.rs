use std::cmp::min;

use cosmwasm_bignumber::math::Decimal256;
use cosmwasm_std::{Deps, Env, Order, StdResult};

use cw_storage_plus::Bound;
use moneymarket::oracle::{
    ConfigResponse, PriceResponse, PricesResponse, PricesResponseElem, SourceInfoResponse,
};

use crate::{
    error::ContractError,
    functions::get_price,
    state::{PriceInfo, ASSETS, CONFIG},
};

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = CONFIG.load(deps.storage)?;
    let resp = ConfigResponse {
        owner: state.owner.to_string(),
        base_asset: state.base_asset,
    };

    Ok(resp)
}

pub fn query_source_info(deps: Deps, asset: String) -> Result<SourceInfoResponse, ContractError> {
    match ASSETS.load(deps.storage, asset.clone()) {
        Ok(source) => Ok(SourceInfoResponse { source }),
        Err(_) => Err(ContractError::AssetIsNotWhitelisted { asset }),
    }
}

pub fn query_price(
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

pub fn query_prices(
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
        .take(curr_limit as usize)
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
