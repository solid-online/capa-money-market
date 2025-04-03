use crate::error::ContractError;
use crate::state::{
    read_borrower_info, read_borrowers, read_config, read_contract_balance_info,
    remove_borrower_info, store_borrower_info, store_contract_balance_info, BorrowerInfo, Config,
    ContractBalanceInfo,
};

use cosmwasm_bignumber::math::Uint256;
use cosmwasm_std::{
    attr, to_binary, Addr, CosmosMsg, Deps, DepsMut, MessageInfo, Response, StdResult, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use moneymarket::custody::{BorrowerResponse, BorrowersResponse};
use moneymarket::liquidation::Cw20HookMsg as LiquidationCw20HookMsg;

/// Deposit new collateral
/// Executor: Collateral token contract
pub fn deposit_collateral(
    deps: DepsMut,
    borrower: Addr,
    amount: Uint256,
) -> Result<Response, ContractError> {
    let borrower_validated = deps.api.addr_validate(borrower.as_str())?;
    let mut borrower_info: BorrowerInfo = read_borrower_info(deps.storage, &borrower_validated);

    let mut contract_balance_info: ContractBalanceInfo = read_contract_balance_info(deps.storage)?;

    // Update borrower LunaX balance
    borrower_info.balance += amount;
    borrower_info.spendable += amount;
    // Update contract LunaX balance
    contract_balance_info.balance += amount;
    // store borrower and balance info
    store_borrower_info(deps.storage, &borrower_validated, &borrower_info)?;
    store_contract_balance_info(deps.storage, &contract_balance_info)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "deposit_collateral"),
        attr("borrower", borrower.as_str()),
        attr("amount", amount.to_string()),
    ]))
}

/// Withdraw spendable collateral or a specified amount of collateral amount is in LUNA
/// Executor: borrower
pub fn withdraw_collateral(
    deps: DepsMut,
    info: MessageInfo,
    amount: Option<Uint256>,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    let mut contract_balance_info: ContractBalanceInfo = read_contract_balance_info(deps.storage)?;
    // get the borrower
    let borrower = info.sender;
    // load borrower info from state
    let mut borrower_info: BorrowerInfo = read_borrower_info(deps.storage, &borrower);
    // query LunaX exchange rate

    // Check spendable balance
    let amount = amount.unwrap_or(borrower_info.spendable);
    // if spenable is less then amount return error
    if borrower_info.spendable < amount {
        return Err(ContractError::WithdrawAmountExceedsSpendable(
            borrower_info.spendable.into(),
        ));
    }

    // decrease borrower collateral spendable and balance
    borrower_info.balance = borrower_info.balance - amount;
    borrower_info.spendable = borrower_info.spendable - amount;
    // Update contract luna balance
    contract_balance_info.balance = contract_balance_info.balance - amount;
    // if the withdrawed all remove borrower_info else store the new values
    if borrower_info.balance == Uint256::zero() {
        remove_borrower_info(deps.storage, &borrower);
    } else {
        store_borrower_info(deps.storage, &borrower, &borrower_info)?;
    }
    // store contract info
    store_contract_balance_info(deps.storage, &contract_balance_info)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.collateral_token.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: borrower.to_string(),
                amount: amount.into(),
            })?,
        }))
        .add_attributes(vec![
            attr("action", "withdraw_collateral"),
            attr("borrower", borrower.as_str()),
            attr("amount", amount.to_string()),
        ]))
}

/// Decrease spendable collateral to lock
/// specified amount of collateral token
/// Executor: overseer
pub fn lock_collateral(
    deps: DepsMut,
    info: MessageInfo,
    borrower: Addr,
    amount: Uint256,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    // only overseer can execute it
    if info.sender != config.overseer_contract {
        return Err(ContractError::Unauthorized {});
    }
    // get the borrower info
    let borrower_validated: Addr = deps.api.addr_validate(borrower.as_str())?;
    let mut borrower_info: BorrowerInfo = read_borrower_info(deps.storage, &borrower_validated);
    // check if the borrower has something spendable to lock, else return an error
    if amount > borrower_info.spendable {
        return Err(ContractError::LockAmountExceedsSpendable(
            borrower_info.spendable.into(),
        ));
    }
    // update spendable amount and store it
    borrower_info.spendable = borrower_info.spendable - amount;
    store_borrower_info(deps.storage, &borrower_validated, &borrower_info)?;
    Ok(Response::new().add_attributes(vec![
        attr("action", "lock_collateral"),
        attr("borrower", borrower),
        attr("amount", amount),
    ]))
}

/// Increase spendable collateral to unlock
/// specified amount of collateral token
/// Executor: overseer
pub fn unlock_collateral(
    deps: DepsMut,
    info: MessageInfo,
    borrower: Addr,
    amount: Uint256,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    // check that the executor is the overseer contract
    if info.sender != config.overseer_contract {
        return Err(ContractError::Unauthorized {});
    }
    // load borrower info
    let borrower_validated: Addr = deps.api.addr_validate(borrower.as_str())?;
    let mut borrower_info: BorrowerInfo = read_borrower_info(deps.storage, &borrower_validated);
    // check how many balance is locked
    let borrowed_amt = borrower_info.balance - borrower_info.spendable;
    // if the amount is greater then the one locked return error else update the borrower_info
    if amount > borrowed_amt {
        return Err(ContractError::UnlockAmountExceedsLocked(
            borrowed_amt.into(),
        ));
    }

    borrower_info.spendable += amount;
    store_borrower_info(deps.storage, &borrower_validated, &borrower_info)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "unlock_collateral"),
        attr("borrower", borrower),
        attr("amount", amount),
    ]))
}

/// Liquidate the collateral using a liquidation queue.
/// can be executed only from overseer contract
pub fn liquidate_collateral(
    deps: DepsMut,
    info: MessageInfo,
    liquidator: Addr,
    borrower: Addr,
    amount: Uint256,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    let mut contract_balance_info: ContractBalanceInfo = read_contract_balance_info(deps.storage)?;

    // Only overseer can execute the contract
    if info.sender != config.overseer_contract {
        return Err(ContractError::Unauthorized {});
    }
    // load borrower info and get the locked balance
    let borrower_validated: Addr = deps.api.addr_validate(borrower.as_str())?;
    let mut borrower_info: BorrowerInfo = read_borrower_info(deps.storage, &borrower_validated);
    let borrowed_amt = borrower_info.balance - borrower_info.spendable;
    // Check that amount is less then amount locked
    if amount > borrowed_amt {
        return Err(ContractError::LiquidationAmountExceedsLocked(
            borrowed_amt.into(),
        ));
    }
    // update borrower balance
    borrower_info.balance = borrower_info.balance - amount;
    contract_balance_info.balance = contract_balance_info.balance - amount;
    store_borrower_info(deps.storage, &borrower_validated, &borrower_info)?;
    store_contract_balance_info(deps.storage, &contract_balance_info)?;
    // ExecuteBid on liquidation contracts sending the token.
    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.collateral_token.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: config.liquidation_contract.to_string(),
                amount: amount.into(),
                msg: to_binary(&LiquidationCw20HookMsg::ExecuteBid {
                    liquidator: liquidator.to_string(),
                    fee_address: Some(config.collector_contract.to_string()),
                    repay_address: Some(config.market_contract.to_string()),
                    borrower_address: Some(borrower_validated.to_string()),
                })?,
            })?,
        }))
        .add_attributes(vec![
            attr("action", "liquidate_collateral"),
            attr("liquidator", liquidator),
            attr("borrower", borrower),
            attr("amount", amount),
        ]))
}

pub fn query_borrower(deps: Deps, borrower: Addr) -> StdResult<BorrowerResponse> {
    let borrower_validated = deps.api.addr_validate(borrower.as_str())?;
    let borrower_info: BorrowerInfo = read_borrower_info(deps.storage, &borrower_validated);
    Ok(BorrowerResponse {
        borrower: borrower.to_string(),
        balance: borrower_info.balance,
        spendable: borrower_info.spendable,
    })
}

pub fn query_borrowers(
    deps: Deps,
    start_after: Option<Addr>,
    limit: Option<u32>,
) -> StdResult<BorrowersResponse> {
    let start_after = if let Some(start_after) = start_after {
        Some(deps.api.addr_validate(start_after.as_str())?)
    } else {
        None
    };

    let borrowers = read_borrowers(deps, start_after, limit)?;
    Ok(BorrowersResponse { borrowers })
}
