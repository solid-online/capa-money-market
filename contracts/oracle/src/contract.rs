use crate::error::ContractError;
use crate::execute::{run_feed_prices, run_register_asset, run_update_config, run_update_source};

use crate::query::{query_config, query_price, query_prices, query_source_info};
use crate::state::{Config, CONFIG};

use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use moneymarket::oracle::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

#[entry_point]
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
            base_asset: msg.base_asset.clone(),
        },
    )?;

    Ok(Response::default()
        .add_attribute("owner", msg.owner)
        .add_attribute("base_asset", msg.base_asset))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterAsset { asset, source } => {
            run_register_asset(deps, env, info, asset, source)
        }
        ExecuteMsg::UpdateConfig { owner } => run_update_config(deps, info, owner),
        ExecuteMsg::UpdateSource { asset, source } => {
            run_update_source(deps, env, info, asset, source)
        }
        ExecuteMsg::FeedPrice { prices } => run_feed_prices(deps, env, info, prices),
    }
}

#[entry_point]
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

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
}
