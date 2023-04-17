use cosmwasm_bignumber::math::{Decimal256, Uint256};
use cosmwasm_std::{
    attr, to_binary, Binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use moneymarket::market::ExecuteMsg;

use crate::{error::ContractError, state::read_config};

const DEFAULT_FLASH_MINT_FEE: Decimal256 = Decimal256::zero();

pub fn flash_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg_callback: Binary,
    amount: Uint256,
) -> Result<Response, ContractError> {
    // Load config
    let config = read_config(deps.storage)?;

    // Compute fee amount
    let flash_mint_fee = config.flash_mint_fee.unwrap_or(DEFAULT_FLASH_MINT_FEE);
    let fee_amount = flash_mint_fee * amount;

    let messages: Vec<CosmosMsg> = vec![
        // Mint
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.stable_contract.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: info.sender.to_string(),
                amount: amount.into(),
            })?,
        }),
        //Callback
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: info.sender.to_string(),
            funds: vec![],
            msg: msg_callback,
        }),
        // Private flash end
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            funds: vec![],
            msg: to_binary(&ExecuteMsg::PrivateFlashEnd {
                flash_minter: info.sender.to_string(),
                burn_amount: amount,
                fee_amount,
            })?,
        }),
    ];

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "flash_mint"),
        attr("flash_minter", info.sender),
        attr("amount", amount),
        attr("fee_amount", fee_amount),
    ]))
}

pub fn private_flash_end(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    flash_minter: String,
    burn_amount: Uint256,
    fee_amount: Uint256,
) -> Result<Response, ContractError> {
    // The sender must be the contract itself
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    let config = read_config(deps.storage)?;

    // Insert msg burn
    let mut messages: Vec<CosmosMsg> = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.stable_contract.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::BurnFrom {
            owner: flash_minter.to_string(),
            amount: burn_amount.into(),
        })?,
    })];

    // Insert msg fee transfer to collector only if fee_amount > 0
    if fee_amount > Uint256::zero() {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.stable_contract.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: flash_minter,
                recipient: config.collector_contract.to_string(),
                amount: fee_amount.into(),
            })?,
        }));
    }

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "private_flash_end"))
}
