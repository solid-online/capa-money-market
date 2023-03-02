use std::ops::{Div, Mul};

use cosmwasm_bignumber::math::{Decimal256, Uint256};
use cosmwasm_std::{
    attr, to_binary, Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use moneymarket::market::{BorrowerInfoResponse, BorrowerInfosResponse};
use moneymarket::oracle::PriceResponse;
use moneymarket::overseer::BorrowLimitResponse;
use moneymarket::querier::{query_price, TimeConstraints};

use crate::error::ContractError;
use crate::querier::query_borrow_limit;
use crate::state::{
    read_borrower_info, read_borrower_infos, read_config, read_state, store_borrower_info,
    store_state, BorrowerInfo, Config, State,
};

pub fn borrow_stable(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    borrow_amount: Uint256,
    to: Option<Addr>,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    let mut state: State = read_state(deps.storage)?;

    let borrower = info.sender;
    let mut liability: BorrowerInfo = read_borrower_info(deps.storage, &borrower);

    // compute fee to borrow
    let one_time_borrow_fee = compute_borrow_fee(deps.as_ref(), &env, &config, borrow_amount)?;

    let borrow_limit_res: BorrowLimitResponse = query_borrow_limit(
        deps.as_ref(),
        config.overseer_contract,
        borrower.clone(),
        Some(env.block.time.seconds()),
    )?;
    let borrow_amount_with_fee = borrow_amount + one_time_borrow_fee;
    // if borrow limit is greater then the total debt plus the new one with the one time fee return error
    if borrow_limit_res.borrow_limit < borrow_amount_with_fee + liability.loan_amount {
        return Err(ContractError::BorrowExceedsLimit(
            borrow_limit_res.borrow_limit.into(),
        ));
    }

    liability.loan_amount += borrow_amount_with_fee;
    liability.loan_amount_without_interest += borrow_amount;

    state.total_liabilities += Decimal256::from_uint256(borrow_amount);
    store_state(deps.storage, &state)?;
    store_borrower_info(deps.storage, &borrower, &liability)?;

    // Mint solid and send to address
    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.stable_contract.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: to.unwrap_or_else(|| borrower.clone()).to_string(),
                amount: borrow_amount.into(),
            })?,
        }))
        .add_attributes(vec![
            attr("action", "borrow_stable"),
            attr("borrower", borrower),
            attr("borrow_amount", borrow_amount),
        ]))
}

// Repay debt, burn the original loan and use the interest to buy CAPA
// loan_amount : repay_amount = loan_amount_without_interest : burn_amount
// burn_amount = (repay_amount * loan_amount_without_interest) / loan_amount
// interest_amount = repay_amount - burn_amount
pub fn repay_stable(
    deps: DepsMut,
    borrower: Addr,
    amount: Uint256,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    // Cannot repay zero amount
    if amount.is_zero() {
        return Err(ContractError::ZeroRepay("Solid".to_string()));
    }

    let mut state: State = read_state(deps.storage)?;

    let borrower_validated = deps.api.addr_validate(borrower.as_str())?;
    let mut liability: BorrowerInfo = read_borrower_info(deps.storage, &borrower_validated);

    let repay_amount: Uint256;
    let burn_amount: Uint256;
    let mut messages: Vec<CosmosMsg> = vec![];
    if liability.loan_amount < amount {
        repay_amount = liability.loan_amount;
        burn_amount = liability.loan_amount_without_interest;
        liability.loan_amount = Uint256::zero();
        liability.loan_amount_without_interest = Uint256::zero();

        // Payback left repay amount to sender
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.stable_contract.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: borrower.to_string(),
                amount: (amount - repay_amount).into(),
            })?,
        }));
    } else {
        repay_amount = amount;
        burn_amount = repay_amount
            .mul(liability.loan_amount_without_interest)
            .div(Decimal256::from_uint256(liability.loan_amount));
        liability.loan_amount = liability.loan_amount - repay_amount;
        liability.loan_amount_without_interest =
            liability.loan_amount_without_interest - burn_amount;
    }

    // BURN Message
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.stable_contract.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Burn {
            amount: burn_amount.into(),
        })?,
    }));

    let interest_amount = repay_amount - burn_amount;

    if !interest_amount.is_zero() {
        // Transfer to collector
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.stable_contract.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: config.collector_contract.to_string(),
                amount: interest_amount.into(),
            })?,
        }));
    }
    state.total_liabilities = state.total_liabilities - Decimal256::from_uint256(burn_amount);
    store_borrower_info(deps.storage, &borrower_validated, &liability)?;
    store_state(deps.storage, &state)?;

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "repay_stable"),
        attr("borrower", borrower),
        attr("repay_amount", repay_amount),
    ]))
}

/// Compute fee to borrow
pub fn compute_borrow_fee(
    deps: Deps,
    env: &Env,
    config: &Config,
    borrow_amount: Uint256,
) -> StdResult<Uint256> {
    let price: PriceResponse = query_price(
        deps,
        config.oracle_contract.clone(),
        config.stable_contract.to_string(),
        "uusd".to_string(),
        Some(TimeConstraints {
            block_time: env.block.time.seconds(),
            valid_timeframe: 1200u64,
        }),
    )?;

    let rate: Decimal256 = if price.rate > Decimal256::one() {
        config.base_borrow_fee
    } else {
        config.base_borrow_fee + (Decimal256::one() - price.rate).div(config.fee_increase_factor)
    };

    let one_time_fee = rate * borrow_amount;

    Ok(one_time_fee)
}

pub fn query_borrower_info(deps: Deps, borrower: Addr) -> StdResult<BorrowerInfoResponse> {
    let borrower_info: BorrowerInfo =
        read_borrower_info(deps.storage, &deps.api.addr_validate(borrower.as_str())?);

    Ok(BorrowerInfoResponse {
        borrower: borrower.to_string(),
        loan_amount: borrower_info.loan_amount,
    })
}

pub fn query_borrower_infos(
    deps: Deps,
    start_after: Option<Addr>,
    limit: Option<u32>,
) -> StdResult<BorrowerInfosResponse> {
    let start_after = if let Some(start_after) = start_after {
        Some(deps.api.addr_validate(start_after.as_str())?)
    } else {
        None
    };

    let borrower_infos: Vec<BorrowerInfoResponse> = read_borrower_infos(deps, start_after, limit)?;
    Ok(BorrowerInfosResponse { borrower_infos })
}
