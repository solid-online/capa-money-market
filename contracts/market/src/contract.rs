#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::borrow::{borrow_stable, query_borrower_info, query_borrower_infos, repay_stable};
use crate::error::ContractError;
use crate::flash_mint::{flash_mint, private_flash_end};
use crate::response::MsgInstantiateContractResponse;
use crate::state::{read_config, read_state, store_config, store_state, Config, State};

use cosmwasm_bignumber::math::Decimal256;
use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw20::{Cw20Coin, Cw20ReceiveMsg, MinterResponse};

use moneymarket::common::optional_addr_validate;
use moneymarket::market::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, StateResponse,
};
use moneymarket::terraswap::InstantiateMsg as TokenInstantiateMsg;
use protobuf::Message;

pub const INITIAL_DEPOSIT_AMOUNT: u128 = 1000000;

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
            contract_addr: deps.api.addr_validate(env.contract.address.as_str())?,
            owner_addr: deps.api.addr_validate(&msg.owner_addr)?,
            stable_contract: Addr::unchecked("".to_string()),
            overseer_contract: Addr::unchecked("".to_string()),
            collector_contract: Addr::unchecked("".to_string()),
            liquidation_contract: Addr::unchecked("".to_string()),
            oracle_contract: Addr::unchecked("".to_string()),
            base_borrow_fee: msg.base_borrow_fee,
            fee_increase_factor: msg.fee_increase_factor,
            flash_mint_fee:msg.fee_flash_mint
        },
    )?;

    store_state(
        deps.storage,
        &State {
            total_liabilities: Decimal256::zero(),
        },
    )?;

    Ok(
        Response::new().add_submessages(vec![SubMsg::reply_on_success(
            CosmosMsg::Wasm(WasmMsg::Instantiate {
                admin: None,
                code_id: msg.stable_code_id,
                funds: vec![],
                label: "stable".to_string(),

                msg: to_binary(&TokenInstantiateMsg {
                    name: "Solid".to_string(),
                    symbol: "SOLID".to_string(),
                    decimals: 6u8,
                    initial_balances: vec![Cw20Coin {
                        address: env.contract.address.to_string(),
                        amount: Uint128::zero(),
                    }],
                    mint: Some(MinterResponse {
                        minter: env.contract.address.to_string(),
                        cap: None,
                    }),
                })?,
            }),
            1,
        )]),
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, info, msg),
        ExecuteMsg::RegisterContracts {
            overseer_contract,
            collector_contract,
            liquidation_contract,
            oracle_contract,
        } => {
            let api = deps.api;
            register_contracts(
                deps,
                info,
                api.addr_validate(&overseer_contract)?,
                api.addr_validate(&collector_contract)?,
                api.addr_validate(&liquidation_contract)?,
                api.addr_validate(&oracle_contract)?,
            )
        }
        ExecuteMsg::UpdateConfig {
            owner_addr,
            liquidation_contract,
            base_borrow_fee,
            fee_increase_factor,
        } => {
            let api = deps.api;
            update_config(
                deps,
                info,
                optional_addr_validate(api, owner_addr)?,
                optional_addr_validate(api, liquidation_contract)?,
                base_borrow_fee,
                fee_increase_factor,
            )
        }
        ExecuteMsg::BorrowStable { borrow_amount, to } => {
            let api = deps.api;
            borrow_stable(
                deps,
                env,
                info,
                borrow_amount,
                optional_addr_validate(api, to)?,
            )
        },

        ExecuteMsg::FlashMint { amount, msg_callback } => {
            flash_mint(
                deps,
                env,
                info,
                msg_callback,
                amount
            )
        },

        ExecuteMsg::PrivateFlashEnd {flash_minter, burn_amount, fee_amount } => {
            private_flash_end(
                deps,
                env,
                info,
                flash_minter,
                burn_amount,
                fee_amount
            )
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

            register_stable(deps, token_addr)
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
        Ok(Cw20HookMsg::RepayStable {}) => {
            let config: Config = read_config(deps.storage)?;
            if contract_addr != config.stable_contract {
                return Err(ContractError::Unauthorized {});
            }

            let cw20_sender_addr = deps.api.addr_validate(&cw20_msg.sender)?;
            repay_stable(deps, cw20_sender_addr, cw20_msg.amount.into())
        }

        Ok(Cw20HookMsg::RepayStableFromLiquidation { borrower }) => {
            let config: Config = read_config(deps.storage)?;
            let cw20_sender_addr = deps.api.addr_validate(&cw20_msg.sender)?;

            if contract_addr != config.stable_contract
                || config.liquidation_contract != cw20_sender_addr
            {
                return Err(ContractError::Unauthorized {});
            }

            let borrower_validated = deps.api.addr_validate(&borrower)?;
            repay_stable(deps, borrower_validated, cw20_msg.amount.into())
        }
        _ => Err(ContractError::MissingRedeemStableHook {}),
    }
}

pub fn register_stable(deps: DepsMut, token_addr: Addr) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;
    if config.stable_contract != Addr::unchecked("".to_string()) {
        return Err(ContractError::Unauthorized {});
    }

    config.stable_contract = deps.api.addr_validate(token_addr.as_str())?;
    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![attr("stable", token_addr)]))
}

pub fn register_contracts(
    deps: DepsMut,
    info: MessageInfo,
    overseer_contract: Addr,
    collector_contract: Addr,
    liquidation_contract: Addr,
    oracle_contract: Addr,
) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;

    if config.owner_addr != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if config.overseer_contract != Addr::unchecked("".to_string())
        || config.collector_contract != Addr::unchecked("".to_string())
        || config.liquidation_contract != Addr::unchecked("".to_string())
        || config.oracle_contract != Addr::unchecked("".to_string())
    {
        return Err(ContractError::Unauthorized {});
    }

    config.overseer_contract = deps.api.addr_validate(overseer_contract.as_str())?;
    config.collector_contract = deps.api.addr_validate(collector_contract.as_str())?;
    config.liquidation_contract = deps.api.addr_validate(liquidation_contract.as_str())?;
    config.oracle_contract = deps.api.addr_validate(oracle_contract.as_str())?;
    store_config(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner_addr: Option<Addr>,
    liquidation_contract: Option<Addr>,
    base_borrow_fee: Option<Decimal256>,
    fee_increase_factor: Option<Decimal256>,
) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;

    // permission check
    if info.sender != config.owner_addr {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner_addr) = owner_addr {
        config.owner_addr = deps.api.addr_validate(owner_addr.as_str())?;
    }

    if let Some(liquidation_contract) = liquidation_contract {
        config.liquidation_contract = deps.api.addr_validate(liquidation_contract.as_str())?;
    }

    if let Some(base_borrow_fee) = base_borrow_fee {
        if base_borrow_fee < Decimal256::one() {
            config.base_borrow_fee = base_borrow_fee
        }
    }

    if let Some(fee_increase_factor) = fee_increase_factor {
        config.fee_increase_factor = fee_increase_factor
    }

    store_config(deps.storage, &config)?;
    Ok(Response::new().add_attributes(vec![attr("action", "update_config")]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::BorrowerInfo { borrower } => to_binary(&query_borrower_info(
            deps,
            deps.api.addr_validate(&borrower)?,
        )?),
        QueryMsg::BorrowerInfos { start_after, limit } => to_binary(&query_borrower_infos(
            deps,
            optional_addr_validate(deps.api, start_after)?,
            limit,
        )?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;
    Ok(ConfigResponse {
        owner_addr: config.owner_addr.to_string(),
        stable_contract: config.stable_contract.to_string(),
        overseer_contract: config.overseer_contract.to_string(),
        collector_contract: config.collector_contract.to_string(),
        liquidation_contract: config.liquidation_contract.to_string(),
        oracle_contract: config.oracle_contract.to_string(),
    })
}

pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state: State = read_state(deps.storage)?;

    Ok(StateResponse {
        total_liabilities: state.total_liabilities,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> StdResult<Response> {

    let config = Config{
        contract_addr: msg.contract_addr,
        owner_addr: msg.owner_addr,
        stable_contract: msg.stable_contract,
        overseer_contract: msg.overseer_contract,
        collector_contract: msg.collector_contract,
        liquidation_contract: msg.liquidation_contract,
        oracle_contract: msg.oracle_contract,
        base_borrow_fee: msg.base_borrow_fee,
        fee_increase_factor: msg.fee_increase_factor,
        flash_mint_fee: msg.flash_mint_fee
    };

    store_config(deps.storage, &config)?;

    Ok(Response::default())
}
