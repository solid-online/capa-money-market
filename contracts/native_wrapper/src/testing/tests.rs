use crate::contract::{execute, instantiate, query, reply};
use crate::error::ContractError;
use crate::response::MsgInstantiateContractResponse;
use crate::state::{store_state, State};
use crate::testing::mock_querier::mock_dependencies;

use cosmwasm_bignumber::math::Decimal256;
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    attr, from_binary, to_binary, Coin, CosmosMsg, Reply, SubMsg, SubMsgResponse, SubMsgResult,
    Uint128, WasmMsg,
};
use cw20::{Cw20Coin, Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use moneymarket::native_wrapper::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse,
};
use moneymarket::terraswap::InstantiateMsg as TokenInstantiateMsg;
use protobuf::Message;

const INITIAL_DEPOSIT_AMOUNT: u128 = 1000000;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "stable".to_string(),
        amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
    }]);

    let msg = InstantiateMsg {
        owner_addr: "owner".to_string(),
        wrapper_code_id: 123u64,
        collateral_denom: "IBCstATOM".to_string(),
        wrapper_denom: "wstAtom".to_string(),
    };

    let info = mock_info("addr0000", &[]);

    let env = mock_env();

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::reply_on_success(
            CosmosMsg::Wasm(WasmMsg::Instantiate {
                admin: None,
                code_id: 123u64,
                funds: vec![],
                label: "IBCstATOM".to_string(),
                msg: to_binary(&TokenInstantiateMsg {
                    name: "wstAtom".to_string(),
                    symbol: "wstAtom".to_string(),
                    decimals: 6u8,
                    initial_balances: vec![Cw20Coin {
                        address: env.contract.address.to_string(),
                        amount: Uint128::zero()
                    }],
                    mint: Some(MinterResponse {
                        minter: MOCK_CONTRACT_ADDR.to_string(),
                        cap: None,
                    }),
                })
                .unwrap(),
            }),
            1
        )]
    );

    let mut token_inst_res = MsgInstantiateContractResponse::new();
    token_inst_res.set_contract_address("statom".to_string());
    let reply_msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(token_inst_res.write_to_bytes().unwrap().into()),
        }),
    };
    let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    deps.querier.with_oracle_price(&[(
        &("statom".to_string(), "IBCstATOM".to_string()),
        &(Decimal256::percent(50), 0u64, 0u64),
    )]);

    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();

    let state: StateResponse = from_binary(&query_res).unwrap();
    assert_eq!(Uint128::zero(), state.total_bond);
    assert_eq!(Uint128::zero(), state.total_supply);
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "stable".to_string(),
        amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
    }]);

    let msg = InstantiateMsg {
        owner_addr: "owner".to_string(),
        wrapper_code_id: 123u64,
        collateral_denom: "IBCstATOM".to_string(),
        wrapper_denom: "wstAtom".to_string(),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let mut token_inst_res = MsgInstantiateContractResponse::new();
    token_inst_res.set_contract_address("statom".to_string());
    let reply_msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(token_inst_res.write_to_bytes().unwrap().into()),
        }),
    };
    let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    // update owner
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        owner_addr: Some("owner1".to_string()),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("owner1".to_string(), config_res.owner_addr);

    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("owner1".to_string(), config_res.owner_addr);
    assert_eq!("statom".to_string(), config_res.wrapper_contract);
    assert_eq!("wstAtom".to_string(), config_res.wrapper_denom);

    // Unauthorized err
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::UpdateConfig { owner_addr: None };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("Must return unauthorized error"),
    }
}

#[test]
fn bond() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner_addr: "owner".to_string(),
        wrapper_code_id: 123u64,
        collateral_denom: "IBCstATOM".to_string(),
        wrapper_denom: "wstAtom".to_string(),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let mut token_inst_res = MsgInstantiateContractResponse::new();
    token_inst_res.set_contract_address("statom".to_string());
    let reply_msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(token_inst_res.write_to_bytes().unwrap().into()),
        }),
    };
    let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "IBCstATOM".to_string(),
            amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        }],
    );

    let env = mock_env();
    let msg = ExecuteMsg::Bond { recipient: None };

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "bond"),
            attr("receiver", "addr0000"),
            attr("mint_amount", "1000000"),
        ]
    );

    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "statom".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: "addr0000".to_string(),
                amount: Uint128::from(1000000u128),
            })
            .unwrap(),
        }))]
    );

    deps.querier.with_oracle_price(&[(
        &("statom".to_string(), "IBCstATOM".to_string()),
        &(
            Decimal256::percent(50),
            env.block.time.seconds(),
            env.block.time.seconds(),
        ),
    )]);

    assert_eq!(
        from_binary::<State>(&query(deps.as_ref(), env.clone(), QueryMsg::State {}).unwrap())
            .unwrap(),
        State {
            total_bond: Uint128::from(1000000u128),
            total_supply: Uint128::from(1000000u128),
            exchange_rate: Uint128::one()
        }
    );

    let msg = ExecuteMsg::Bond {
        recipient: Some("addr0001".to_string()),
    };

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "bond"),
            attr("receiver", "addr0001"),
            attr("mint_amount", "1000000"),
        ]
    );

    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "statom".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: "addr0001".to_string(),
                amount: Uint128::from(1000000u128),
            })
            .unwrap(),
        }))]
    );

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "IBCstAtom".to_string(),
            amount: Uint128::zero(),
        }],
    );
    // Cannot bond with zero funds
    let msg = ExecuteMsg::Bond { recipient: None };
    let res = execute(deps.as_mut(), env.clone(), info, msg);
    let _denom = "IBCstAtom".to_string();
    match res {
        Err(ContractError::ZeroDeposit(_denom)) => (),
        _ => panic!("DO NOT ENTER HERE"),
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "IBCnotAtom".to_string(),
            amount: Uint128::from(1u128),
        }],
    );
    // Cannot bond different denom
    let msg = ExecuteMsg::Bond { recipient: None };
    let res = execute(deps.as_mut(), env, info, msg);
    match res {
        Err(ContractError::ZeroDeposit(_denom)) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn unbound() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner_addr: "owner".to_string(),
        wrapper_code_id: 123u64,
        collateral_denom: "IBCstATOM".to_string(),
        wrapper_denom: "wstAtom".to_string(),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let mut token_inst_res = MsgInstantiateContractResponse::new();
    token_inst_res.set_contract_address("statom".to_string());
    let reply_msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(token_inst_res.write_to_bytes().unwrap().into()),
        }),
    };
    let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    let env = mock_env();

    store_state(
        deps.as_mut().storage,
        &State {
            total_bond: Uint128::zero(),
            total_supply: Uint128::zero(),
            exchange_rate: Uint128::one(),
        },
    )
    .unwrap();

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "IBCstATOM".to_string(),
            amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        }],
    );

    let msg = ExecuteMsg::Bond { recipient: None };

    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = mock_info("solid", &[]);
    // Wrong cw20 token sent
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0001".to_string(),
        amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        msg: to_binary(&Cw20HookMsg::Unbound { recipient: None }).unwrap(),
    });

    let res = execute(deps.as_mut(), env.clone(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let info = mock_info("statom", &[]);
    // Zero repay amount
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(0u128),
        msg: to_binary(&Cw20HookMsg::Unbound { recipient: None }).unwrap(),
    });

    let _st_atom = "wstAtom";
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    match res {
        Err(ContractError::ZeroRepay(_st_atom)) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0001".to_string(),
        amount: Uint128::from(100000u128),
        msg: to_binary(&Cw20HookMsg::Unbound { recipient: None }).unwrap(),
    });

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "redeem_collateral"),
            attr("burn_amount", "100000"),
            attr("receiver", "addr0001"),
            attr("redeem_amount", "100000"),
        ]
    );

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0001".to_string(),
        amount: Uint128::from(100000u128),
        msg: to_binary(&Cw20HookMsg::Unbound {
            recipient: Some("addr0002".to_string()),
        })
        .unwrap(),
    });

    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "redeem_collateral"),
            attr("burn_amount", "100000"),
            attr("receiver", "addr0002"),
            attr("redeem_amount", "100000"),
        ]
    );
}
