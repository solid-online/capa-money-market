#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::bond::{bond, unbound};
use crate::error::ContractError;
use crate::response::MsgInstantiateContractResponse;
use crate::state::{read_config, read_state, store_config, store_state, Config, State};

use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw20::{Cw20Coin, Cw20ReceiveMsg, MinterResponse};

use moneymarket::common::optional_addr_validate;
use moneymarket::native_wrapper::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, StateResponse,
};
use moneymarket::terraswap::InstantiateMsg as TokenInstantiateMsg;
use protobuf::Message;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    store_config(
        deps.storage,
        &Config {
            owner_addr: deps.api.addr_validate(&msg.owner_addr)?,
            collateral_denom: msg.collateral_denom.clone(),
            wrapper_denom: msg.wrapper_denom.clone(),
            wrapper_contract: Addr::unchecked(""),
        },
    )?;

    store_state(
        deps.storage,
        &State {
            total_bond: Uint128::zero(),
            total_supply: Uint128::zero(),
        },
    )?;

    Ok(
        Response::new().add_submessages(vec![SubMsg::reply_on_success(
            CosmosMsg::Wasm(WasmMsg::Instantiate {
                admin: None,
                code_id: msg.wrapper_code_id,
                funds: vec![],
                label: msg.collateral_denom,

                // TODO check symbol
                msg: to_binary(&TokenInstantiateMsg {
                    name: msg.wrapper_denom.clone(),
                    symbol: msg.wrapper_denom,
                    decimals: 6u8,
                    mint: Some(MinterResponse {
                        minter: env.contract.address.to_string(),
                        cap: None,
                    }),
                    initial_balances: vec![Cw20Coin {
                        address: env.contract.address.to_string(),
                        amount: Uint128::zero(),
                    }],
                })?,
            }),
            1,
        )]),
    )
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

        ExecuteMsg::UpdateConfig { owner_addr } => {
            let api = deps.api;
            update_config(deps, info, optional_addr_validate(api, owner_addr)?)
        }
        ExecuteMsg::Bond { recipient } => {
            let api = deps.api;
            bond(deps, info, optional_addr_validate(api, recipient)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        1 => {
            // get new token's contract address
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(
                msg.result.unwrap().data.unwrap().as_slice(),
            )
            .map_err(|_| {
                ContractError::Std(StdError::parse_err(
                    "MsgInstantiateContractResponse",
                    "failed to parse data",
                ))
            })?;
            let token_addr = Addr::unchecked(res.get_contract_address());

            register_wrapper(deps, token_addr)
        }
        _ => Err(ContractError::InvalidReplyId {}),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let contract_addr = info.sender;
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Unbound { recipient }) => {
            let config: Config = read_config(deps.storage)?;
            if contract_addr != config.wrapper_contract {
                return Err(ContractError::Unauthorized {});
            }

            let receiver = deps
                .api
                .addr_validate(recipient.unwrap_or(cw20_msg.sender).as_str())?;
            unbound(deps, receiver, cw20_msg.amount)
        }
        _ => Err(ContractError::MissingRedeemStableHook {}),
    }
}

pub fn register_wrapper(deps: DepsMut, token_addr: Addr) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;
    if config.wrapper_contract != Addr::unchecked("") {
        return Err(ContractError::Unauthorized {});
    }

    config.wrapper_contract = deps.api.addr_validate(token_addr.as_str())?;
    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![attr("wrapper", token_addr)]))
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner_addr: Option<Addr>,
) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;

    // permission check
    if info.sender.as_str() != config.owner_addr {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner_addr) = owner_addr {
        config.owner_addr = deps.api.addr_validate(owner_addr.as_str())?;
    }

    store_config(deps.storage, &config)?;
    Ok(Response::new().add_attributes(vec![attr("action", "update_config")]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;
    Ok(ConfigResponse {
        owner_addr: config.owner_addr.to_string(),
        collateral_denom: config.collateral_denom.to_string(),
        wrapper_denom: config.wrapper_denom,
        wrapper_contract: config.wrapper_contract.to_string(),
    })
}

pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state: State = read_state(deps.storage)?;

    Ok(StateResponse {
        total_bond: state.total_bond,
        total_supply: state.total_supply,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
