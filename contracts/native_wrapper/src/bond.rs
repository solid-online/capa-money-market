use std::ops::Sub;

use crate::error::ContractError;
use crate::state::{read_config, read_state, store_state, Config, State};
use cosmwasm_std::{
    attr, to_binary, Addr, BankMsg, CosmosMsg, DepsMut, MessageInfo, Response, WasmMsg,
};
use cosmwasm_std::{Coin, Uint128};
use cw20::Cw20ExecuteMsg;

pub fn bond(
    deps: DepsMut,
    info: MessageInfo,
    recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    let mut state: State = read_state(deps.storage)?;

    if info.funds.len() != 1 {
        return Err(ContractError::TooManyCoins {});
    }
    // Check base denom bond
    let amount: Uint128 = info
        .funds
        .iter()
        .find(|c| c.denom == config.collateral_denom)
        .map(|c| c.amount)
        .unwrap_or_else(Uint128::zero);

    // Cannot bond zero amount
    if amount.is_zero() {
        return Err(ContractError::ZeroDeposit(config.collateral_denom));
    }

    let receiver = recipient.unwrap_or(info.sender);

    state.total_bond += amount;
    state.total_supply += amount;
    store_state(deps.storage, &state)?;

    // Mint with 1:1 ratio
    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.wrapper_contract.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: receiver.to_string(),
                amount,
            })?,
        }))
        .add_attributes(vec![
            attr("action", "bond"),
            attr("receiver", receiver),
            attr("mint_amount", amount),
        ]))
}

// Repay debt, burn the wrapper and send the native token with 1:1 ratio
pub fn unbound(deps: DepsMut, receiver: Addr, amount: Uint128) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    let mut state: State = read_state(deps.storage)?;

    // Cannot bond zero amount
    if amount.is_zero() {
        return Err(ContractError::ZeroRepay(config.wrapper_denom));
    }

    state.total_bond = state.total_bond.sub(amount);
    state.total_supply = state.total_supply.sub(amount);
    store_state(deps.storage, &state)?;

    // Burn with 1:1 Ratio
    Ok(Response::new()
        .add_messages(vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.wrapper_contract.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Burn { amount })?,
            }),
            CosmosMsg::Bank(BankMsg::Send {
                to_address: receiver.to_string(),
                amount: vec![Coin {
                    denom: config.collateral_denom,
                    amount,
                }],
            }),
        ])
        .add_attributes(vec![
            attr("action", "redeem_collateral"),
            attr("burn_amount", amount),
            attr("receiver", receiver),
            attr("redeem_amount", amount),
        ]))
}
