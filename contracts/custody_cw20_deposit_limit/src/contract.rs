use cosmwasm_bignumber::math::Uint256;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult,
};

use crate::collateral::{
    deposit_collateral, liquidate_collateral, lock_collateral, query_borrower, query_borrowers,
    unlock_collateral, withdraw_collateral,
};
use crate::error::ContractError;
use crate::state::{
    read_config, read_contract_balance_info, store_config, store_contract_balance_info, Config,
    ContractBalanceInfo,
};

use cw20::Cw20ReceiveMsg;
use moneymarket::common::optional_addr_validate;
use moneymarket::custody_deposit_cap::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        owner: deps.api.addr_validate(&msg.owner)?,
        overseer_contract: deps.api.addr_validate(&msg.overseer_contract)?,
        collateral_token: deps.api.addr_validate(&msg.collateral_token)?,
        market_contract: deps.api.addr_validate(&msg.market_contract)?,
        liquidation_contract: deps.api.addr_validate(&msg.liquidation_contract)?,
        collector_contract: deps.api.addr_validate(&msg.collector_contract)?,
        max_deposit: msg.max_deposit,
    };

    let contract_balance_info = ContractBalanceInfo {
        balance: Uint256::zero(),
    };

    store_contract_balance_info(deps.storage, &contract_balance_info)?;
    store_config(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, info, msg),
        ExecuteMsg::UpdateConfig {
            owner,
            liquidation_contract,
            collector_contract,
            max_deposit,
        } => {
            let api = deps.api;
            update_config(
                deps,
                info,
                optional_addr_validate(api, owner)?,
                optional_addr_validate(api, liquidation_contract)?,
                optional_addr_validate(api, collector_contract)?,
                max_deposit,
            )
        }
        ExecuteMsg::LockCollateral { borrower, amount } => {
            let borrower_addr = deps.api.addr_validate(&borrower)?;
            lock_collateral(deps, info, borrower_addr, amount)
        }
        ExecuteMsg::UnlockCollateral { borrower, amount } => {
            let borrower_addr = deps.api.addr_validate(&borrower)?;
            unlock_collateral(deps, info, borrower_addr, amount)
        }
        ExecuteMsg::WithdrawCollateral { amount } => withdraw_collateral(deps, info, amount),
        ExecuteMsg::LiquidateCollateral {
            liquidator,
            borrower,
            amount,
        } => {
            let liquidator_addr = deps.api.addr_validate(&liquidator)?;
            let borrower_addr = deps.api.addr_validate(&borrower)?;
            liquidate_collateral(deps, info, liquidator_addr, borrower_addr, amount)
        }
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let contract_addr = info.sender;
    let contract_balance = read_contract_balance_info(deps.storage)?;

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::DepositCollateral {}) => {
            // only asset contract can execute this message
            let config: Config = read_config(deps.storage)?;
            if contract_addr != config.collateral_token {
                return Err(ContractError::Unauthorized {});
            }

            // check if the contract balance is less than max deposit
            let new_balance = contract_balance.balance + cw20_msg.amount.into();

            if new_balance > config.max_deposit {
                return Err(ContractError::InvalidMaxDeposit(new_balance.into()));
            }

            let cw20_sender_addr = deps.api.addr_validate(&cw20_msg.sender)?;
            deposit_collateral(deps, cw20_sender_addr, cw20_msg.amount.into())
        }
        _ => Err(ContractError::MissingDepositCollateralHook {}),
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<Addr>,
    liquidation_contract: Option<Addr>,
    collector_contract: Option<Addr>,
    max_deposit: Option<Uint256>,
) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;
    let contract_balance = read_contract_balance_info(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(owner.as_str())?;
    }

    if let Some(liquidation_contract) = liquidation_contract {
        config.liquidation_contract = deps.api.addr_validate(liquidation_contract.as_str())?;
    }

    if let Some(collector_contract) = collector_contract {
        config.collector_contract = deps.api.addr_validate(collector_contract.as_str())?;
    }

    if let Some(max_deposit) = max_deposit {
        if contract_balance.balance > max_deposit {
            return Err(ContractError::InvalidMaxDeposit(
                contract_balance.balance.into(),
            ));
        }
        config.max_deposit = max_deposit;
    }

    store_config(deps.storage, &config)?;
    Ok(Response::new().add_attributes(vec![attr("action", "update_config")]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Borrower { address } => {
            let addr = deps.api.addr_validate(&address)?;
            to_binary(&query_borrower(deps, addr)?)
        }
        QueryMsg::Borrowers { start_after, limit } => to_binary(&query_borrowers(
            deps,
            optional_addr_validate(deps.api, start_after)?,
            limit,
        )?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        collateral_token: config.collateral_token.to_string(),
        overseer_contract: config.overseer_contract.to_string(),
        market_contract: config.market_contract.to_string(),
        liquidation_contract: config.liquidation_contract.to_string(),
        collector_contract: config.collector_contract.to_string(),
        max_deposit: config.max_deposit,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
