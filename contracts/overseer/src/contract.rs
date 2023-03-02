#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::collateral::{
    liquidate_collateral, lock_collateral, query_all_collaterals, query_borrow_limit,
    query_collaterals, unlock_collateral,
};
use crate::error::ContractError;

use crate::state::{
    read_config, read_whitelist, read_whitelist_elem, store_config, store_whitelist_elem, Config,
    WhitelistElem,
};

use cosmwasm_bignumber::math::Decimal256;
use moneymarket::common::optional_addr_validate;
use moneymarket::overseer::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, WhitelistResponse,
    WhitelistResponseElem,
};

pub const BLOCKS_PER_YEAR: u128 = 4656810;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    store_config(
        deps.storage,
        &Config {
            owner_addr: deps.api.addr_validate(&msg.owner_addr)?,
            oracle_contract: deps.api.addr_validate(&msg.oracle_contract)?,
            market_contract: deps.api.addr_validate(&msg.market_contract)?,
            liquidation_contract: deps.api.addr_validate(&msg.liquidation_contract)?,
            collector_contract: deps.api.addr_validate(&msg.collector_contract)?,
            stable_contract: deps.api.addr_validate(&msg.stable_contract)?,
            price_timeframe: msg.price_timeframe,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
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
        ExecuteMsg::UpdateConfig {
            owner_addr,
            oracle_contract,
            liquidation_contract,
            price_timeframe,
        } => {
            let api = deps.api;
            update_config(
                deps,
                info,
                optional_addr_validate(api, owner_addr)?,
                optional_addr_validate(api, oracle_contract)?,
                optional_addr_validate(api, liquidation_contract)?,
                price_timeframe,
            )
        }
        ExecuteMsg::Whitelist {
            name,
            symbol,
            collateral_token,
            custody_contract,
            max_ltv,
        } => {
            let api = deps.api;
            register_whitelist(
                deps,
                info,
                name,
                symbol,
                api.addr_validate(&collateral_token)?,
                api.addr_validate(&custody_contract)?,
                max_ltv,
            )
        }
        ExecuteMsg::UpdateWhitelist {
            collateral_token,
            custody_contract,
            max_ltv,
        } => {
            let api = deps.api;
            update_whitelist(
                deps,
                info,
                api.addr_validate(&collateral_token)?,
                optional_addr_validate(api, custody_contract)?,
                max_ltv,
            )
        }
        ExecuteMsg::LockCollateral { collaterals } => lock_collateral(deps, info, collaterals),
        ExecuteMsg::UnlockCollateral { collaterals } => {
            unlock_collateral(deps, env, info, collaterals)
        }
        ExecuteMsg::LiquidateCollateral { borrower } => {
            let api = deps.api;
            liquidate_collateral(deps, env, info, api.addr_validate(&borrower)?)
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner_addr: Option<Addr>,
    oracle_contract: Option<Addr>,
    liquidation_contract: Option<Addr>,
    price_timeframe: Option<u64>,
) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;

    if info.sender != config.owner_addr {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner_addr) = owner_addr {
        config.owner_addr = deps.api.addr_validate(owner_addr.as_ref())?;
    }

    if let Some(oracle_contract) = oracle_contract {
        config.oracle_contract = deps.api.addr_validate(oracle_contract.as_ref())?;
    }

    if let Some(liquidation_contract) = liquidation_contract {
        config.liquidation_contract = deps.api.addr_validate(liquidation_contract.as_ref())?;
    }

    if let Some(price_timeframe) = price_timeframe {
        config.price_timeframe = price_timeframe;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![attr("action", "update_config")]))
}

pub fn register_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    symbol: String,
    collateral_token: Addr,
    custody_contract: Addr,
    max_ltv: Decimal256,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    if info.sender != config.owner_addr {
        return Err(ContractError::Unauthorized {});
    }

    let collateral_token_validated = deps.api.addr_validate(collateral_token.as_str())?;
    if read_whitelist_elem(deps.storage, &collateral_token_validated).is_ok() {
        return Err(ContractError::TokenAlreadyRegistered {});
    }

    if max_ltv <= Decimal256::zero() || max_ltv >= Decimal256::from_ratio(100, 1) {
        return Err(ContractError::InvalidMaxLtv {});
    }
    store_whitelist_elem(
        deps.storage,
        &collateral_token_validated,
        &WhitelistElem {
            name: name.to_string(),
            symbol: symbol.to_string(),
            custody_contract: deps.api.addr_validate(custody_contract.as_str())?,
            max_ltv,
        },
    )?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "register_whitelist"),
        attr("name", name),
        attr("symbol", symbol),
        attr("collateral_token", collateral_token),
        attr("custody_contract", custody_contract),
        attr("LTV", max_ltv.to_string()),
    ]))
}

pub fn update_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    collateral_token: Addr,
    custody_contract: Option<Addr>,
    max_ltv: Option<Decimal256>,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    if info.sender != config.owner_addr {
        return Err(ContractError::Unauthorized {});
    }

    let collateral_token_validated = deps.api.addr_validate(collateral_token.as_str())?;
    let mut whitelist_elem: WhitelistElem =
        read_whitelist_elem(deps.storage, &collateral_token_validated)?;

    if let Some(custody_contract) = custody_contract {
        whitelist_elem.custody_contract = deps.api.addr_validate(custody_contract.as_str())?;
    }

    if let Some(max_ltv) = max_ltv {
        if max_ltv <= Decimal256::zero() || max_ltv >= Decimal256::from_ratio(100, 1) {
            return Err(ContractError::InvalidMaxLtv {});
        }

        whitelist_elem.max_ltv = max_ltv;
    }

    store_whitelist_elem(deps.storage, &collateral_token_validated, &whitelist_elem)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "update_whitelist"),
        attr("collateral_token", collateral_token),
        attr("custody_contract", whitelist_elem.custody_contract),
        attr("LTV", whitelist_elem.max_ltv.to_string()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Whitelist {
            collateral_token,
            start_after,
            limit,
        } => to_binary(&query_whitelist(
            deps,
            optional_addr_validate(deps.api, collateral_token)?,
            optional_addr_validate(deps.api, start_after)?,
            limit,
        )?),
        QueryMsg::Collaterals { borrower } => to_binary(&query_collaterals(
            deps,
            deps.api.addr_validate(&borrower)?,
        )?),
        QueryMsg::AllCollaterals { start_after, limit } => to_binary(&query_all_collaterals(
            deps,
            optional_addr_validate(deps.api, start_after)?,
            limit,
        )?),
        QueryMsg::BorrowLimit {
            borrower,
            block_time,
        } => to_binary(&query_borrow_limit(
            deps,
            deps.api.addr_validate(&borrower)?,
            block_time,
        )?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;
    Ok(ConfigResponse {
        owner_addr: config.owner_addr.to_string(),
        oracle_contract: config.oracle_contract.to_string(),
        market_contract: config.market_contract.to_string(),
        liquidation_contract: config.liquidation_contract.to_string(),
        collector_contract: config.collector_contract.to_string(),
        stable_contract: config.stable_contract.to_string(),
        price_timeframe: config.price_timeframe,
    })
}

pub fn query_whitelist(
    deps: Deps,
    collateral_token: Option<Addr>,
    start_after: Option<Addr>,
    limit: Option<u32>,
) -> StdResult<WhitelistResponse> {
    if let Some(collateral_token) = collateral_token {
        let whitelist_elem: WhitelistElem = read_whitelist_elem(
            deps.storage,
            &deps.api.addr_validate(collateral_token.as_str())?,
        )?;
        Ok(WhitelistResponse {
            elems: vec![WhitelistResponseElem {
                name: whitelist_elem.name,
                symbol: whitelist_elem.symbol,
                max_ltv: whitelist_elem.max_ltv,
                custody_contract: whitelist_elem.custody_contract.to_string(),
                collateral_token: collateral_token.to_string(),
            }],
        })
    } else {
        let start_after = if let Some(start_after) = start_after {
            Some(deps.api.addr_validate(start_after.as_str())?)
        } else {
            None
        };

        let whitelist: Vec<WhitelistResponseElem> = read_whitelist(deps, start_after, limit)?;
        Ok(WhitelistResponse { elems: whitelist })
    }
}
