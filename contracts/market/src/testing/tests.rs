use crate::borrow::borrow_stable as _borrow_stable;
use crate::contract::{execute, instantiate, migrate, query, reply, INITIAL_DEPOSIT_AMOUNT};
use crate::error::ContractError;
use crate::response::MsgInstantiateContractResponse;
use crate::state::{read_borrower_infos, read_state, store_state, State};
use crate::testing::mock_querier::mock_dependencies;

use cosmwasm_bignumber::math::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, Coin, CosmosMsg, Reply, SubMsg, SubMsgResponse,
    SubMsgResult, Uint128, WasmMsg,
};
use cw20::{Cw20Coin, Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use moneymarket::market::{
    BorrowerInfoResponse, ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg,
    QueryMsg, StateResponse,
};
use moneymarket::terraswap::InstantiateMsg as TokenInstantiateMsg;
use protobuf::Message;
use std::str::FromStr;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "solid".to_string(),
        amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
    }]);

    let env = mock_env();

    let msg = InstantiateMsg {
        owner_addr: "owner".to_string(),
        stable_code_id: 123u64,
        base_borrow_fee: Decimal256::from_str("0.05").unwrap(),
        fee_increase_factor: Decimal256::from_str("2").unwrap(),
        fee_flash_mint: Decimal256::from_str("0.00025").unwrap(),
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "solid".to_string(),
            amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        }],
    );

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::reply_on_success(
            CosmosMsg::Wasm(WasmMsg::Instantiate {
                admin: None,
                code_id: 123u64,
                funds: vec![],
                label: "stable".to_string(),
                msg: to_binary(&TokenInstantiateMsg {
                    name: "Solid".to_string(),
                    symbol: "SOLID".to_string(),
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

    // Register solid token contract
    let mut token_inst_res = MsgInstantiateContractResponse::new();
    token_inst_res.set_contract_address("solid".to_string());
    let reply_msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(token_inst_res.write_to_bytes().unwrap().into()),
        }),
    };
    let _res = reply(deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();

    // Cannot register again
    let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap_err();

    // Register overseer contract
    let msg = ExecuteMsg::RegisterContracts {
        overseer_contract: "overseer".to_string(),
        collector_contract: "collector".to_string(),
        liquidation_contract: "liquidation".to_string(),
        oracle_contract: "oracle".to_string(),
    };
    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Cannot register again
    let msg = ExecuteMsg::RegisterContracts {
        overseer_contract: "overseer".to_string(),
        collector_contract: "collector".to_string(),
        liquidation_contract: "liquidation".to_string(),
        oracle_contract: "oracle".to_string(),
    };
    let info = mock_info("addr0000", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_binary(&query_res).unwrap();
    assert_eq!("owner".to_string(), config_res.owner_addr);
    assert_eq!("solid".to_string(), config_res.stable_contract);
    assert_eq!("liquidation".to_string(), config_res.liquidation_contract);
    assert_eq!("collector".to_string(), config_res.collector_contract);
    assert_eq!("overseer".to_string(), config_res.overseer_contract);
    assert_eq!("oracle".to_string(), config_res.oracle_contract);

    deps.querier.with_oracle_price(&[(
        &("solid".to_string(), "uusd".to_string()),
        &(
            Decimal256::percent(50),
            env.block.time.seconds(),
            env.block.time.seconds(),
        ),
    )]);

    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();

    let state: StateResponse = from_binary(&query_res).unwrap();
    assert_eq!(Decimal256::zero(), state.total_liabilities);
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "solid".to_string(),
        amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
    }]);

    let env = mock_env();
    deps.querier
        .with_borrow_rate(&[(&"interest".to_string(), &Decimal256::percent(1))]);

    deps.querier.with_oracle_price(&[(
        &("solid".to_string(), "uusd".to_string()),
        &(
            Decimal256::percent(50),
            env.block.time.seconds(),
            env.block.time.seconds(),
        ),
    )]);

    let msg = InstantiateMsg {
        owner_addr: "owner".to_string(),
        stable_code_id: 123u64,
        base_borrow_fee: Decimal256::from_str("0.05").unwrap(),
        fee_increase_factor: Decimal256::from_str("2").unwrap(),
        fee_flash_mint: Decimal256::from_str("0.00025").unwrap(),
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "solid".to_string(),
            amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        }],
    );

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Register solid token contract
    let mut token_inst_res = MsgInstantiateContractResponse::new();
    token_inst_res.set_contract_address("solid".to_string());
    let reply_msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(token_inst_res.write_to_bytes().unwrap().into()),
        }),
    };
    let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    // Register overseer contract
    let msg = ExecuteMsg::RegisterContracts {
        overseer_contract: "overseer".to_string(),
        collector_contract: "collector".to_string(),
        liquidation_contract: "liquidation".to_string(),
        oracle_contract: "oracle".to_string(),
    };
    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // update owner
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        owner_addr: Some("owner1".to_string()),
        liquidation_contract: None,
        base_borrow_fee: Some(Decimal256::from_str("0.006").unwrap()),
        fee_increase_factor: Some(Decimal256::from_str("2").unwrap()),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("owner1".to_string(), config_res.owner_addr);

    // update left items
    let info = mock_info("owner1", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        owner_addr: None,
        liquidation_contract: Some("liquidation2".to_string()),
        base_borrow_fee: None,
        fee_increase_factor: None,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("owner1".to_string(), config_res.owner_addr);
    assert_eq!("liquidation2".to_string(), config_res.liquidation_contract);

    // Unauthorized err
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        owner_addr: None,
        liquidation_contract: None,
        base_borrow_fee: None,
        fee_increase_factor: None,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("Must return unauthorized error"),
    }
}

#[test]
fn borrow_stable() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "solid".to_string(),
        amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
    }]);

    let msg = InstantiateMsg {
        owner_addr: "owner".to_string(),
        stable_code_id: 123u64,
        base_borrow_fee: Decimal256::from_str("0.005").unwrap(),
        fee_increase_factor: Decimal256::from_str("2").unwrap(),
        fee_flash_mint: Decimal256::from_str("0.00025").unwrap(),
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "solid".to_string(),
            amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        }],
    );

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Register solid token contract
    let mut token_inst_res = MsgInstantiateContractResponse::new();
    token_inst_res.set_contract_address("solid".to_string());
    let reply_msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(token_inst_res.write_to_bytes().unwrap().into()),
        }),
    };
    let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    // Register overseer contract
    let msg = ExecuteMsg::RegisterContracts {
        overseer_contract: "overseer".to_string(),
        collector_contract: "collector".to_string(),
        liquidation_contract: "liquidation".to_string(),
        oracle_contract: "oracle".to_string(),
    };
    let mut env = mock_env();
    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // No interest
    deps.querier.with_oracle_price(&[(
        &("solid".to_string(), "uusd".to_string()),
        &(
            Decimal256::one(),
            env.block.time.seconds(),
            env.block.time.seconds(),
        ),
    )]);

    store_state(
        deps.as_mut().storage,
        &State {
            total_liabilities: Decimal256::from_uint256(1000000u128),
        },
    )
    .unwrap();

    env.block.height += 100;

    deps.querier
        .with_borrow_limit(&[(&"addr0000".to_string(), &Uint256::from(1000000u64))]);

    // borrow with 0.005 fee
    deps.querier.with_oracle_price(&[(
        &("solid".to_string(), "uusd".to_string()),
        &(
            Decimal256::one(),
            env.block.time.seconds(),
            env.block.time.seconds(),
        ),
    )]);

    let msg = ExecuteMsg::BorrowStable {
        borrow_amount: Uint256::from(500000u64),
        to: None,
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "borrow_stable"),
            attr("borrower", "addr0000"),
            attr("borrow_amount", "500000")
        ]
    );

    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "solid".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: "addr0000".to_string(),
                amount: Uint128::from(500000u128),
            })
            .unwrap(),
        }))]
    );

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::BorrowerInfo {
            borrower: "addr0000".to_string(),
        },
    )
    .unwrap();
    let liability: BorrowerInfoResponse = from_binary(&res).unwrap();
    assert_eq!(
        liability,
        BorrowerInfoResponse {
            borrower: "addr0000".to_string(),
            loan_amount: Uint256::from(502500u64),
        }
    );

    deps.querier.with_oracle_price(&[(
        &("solid".to_string(), "uusd".to_string()),
        &(
            Decimal256::from_str("1.02").unwrap(),
            env.block.time.seconds(),
            env.block.time.seconds(),
        ),
    )]);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::BorrowerInfo {
            borrower: "addr0000".to_string(),
        },
    )
    .unwrap();

    let borrower_info: BorrowerInfoResponse = from_binary(&res).unwrap();
    assert_eq!(
        borrower_info,
        BorrowerInfoResponse {
            borrower: "addr0000".to_string(),
            loan_amount: Uint256::from(502500u64),
        }
    );

    // Interest rate drop again 0.5% fee
    deps.querier.with_oracle_price(&[(
        &("solid".to_string(), "uusd".to_string()),
        &(
            Decimal256::one(),
            env.block.time.seconds(),
            env.block.time.seconds(),
        ),
    )]);
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::BorrowerInfo {
            borrower: "addr0000".to_string(),
        },
    )
    .unwrap();

    let borrower_info: BorrowerInfoResponse = from_binary(&res).unwrap();
    assert_eq!(
        borrower_info,
        BorrowerInfoResponse {
            borrower: "addr0000".to_string(),
            loan_amount: Uint256::from(502500u128),
        }
    );

    // Cannot borrow more than borrow limit
    let msg = ExecuteMsg::BorrowStable {
        borrow_amount: Uint256::from(500001u64),
        to: None,
    };
    let res = execute(deps.as_mut(), env, info, msg);
    match res {
        Err(ContractError::BorrowExceedsLimit(1000000)) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn repay_stable() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "solid".to_string(),
        amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
    }]);

    let msg = InstantiateMsg {
        owner_addr: "owner".to_string(),
        stable_code_id: 123u64,
        base_borrow_fee: Decimal256::from_str("0.005").unwrap(),
        fee_increase_factor: Decimal256::from_str("2").unwrap(),
        fee_flash_mint: Decimal256::from_str("0.00025").unwrap(),
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "solid".to_string(),
            amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        }],
    );

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    // Register solid token contract
    let mut token_inst_res = MsgInstantiateContractResponse::new();
    token_inst_res.set_contract_address("solid".to_string());
    let reply_msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(token_inst_res.write_to_bytes().unwrap().into()),
        }),
    };
    let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    // Register overseer contract
    let msg = ExecuteMsg::RegisterContracts {
        overseer_contract: "overseer".to_string(),
        collector_contract: "collector".to_string(),
        liquidation_contract: "liquidation".to_string(),
        oracle_contract: "oracle".to_string(),
    };
    let mut env = mock_env();
    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    deps.querier
        .with_borrow_limit(&[(&"addr0000".to_string(), &Uint256::from(1000000u64))]);
    // 0.005 fee to borrow 0.015 repay fee
    deps.querier.with_oracle_price(&[(
        &("solid".to_string(), "uusd".to_string()),
        &(
            Decimal256::from_str("1.03").unwrap(),
            env.block.time.seconds(),
            env.block.time.seconds(),
        ),
    )]);

    store_state(
        deps.as_mut().storage,
        &State {
            total_liabilities: Decimal256::from_uint256(1000000u128),
        },
    )
    .unwrap();

    let msg = ExecuteMsg::BorrowStable {
        borrow_amount: Uint256::from(500000u64),
        to: None,
    };

    env.block.height += 100;
    let info = mock_info("addr0000", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = mock_info("capa", &[]);
    // Wrong cw20 token sent
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(110u128),
        msg: to_binary(&Cw20HookMsg::RepayStable {}).unwrap(),
    });

    let res = execute(deps.as_mut(), env.clone(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let info = mock_info("solid", &[]);
    // Zero repay amount
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(0u128),
        msg: to_binary(&Cw20HookMsg::RepayStable {}).unwrap(),
    });

    let _solid_string = "Solid";
    let res2 = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    match res2 {
        Err(ContractError::ZeroRepay(_solid_string)) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(100000u128),
        msg: to_binary(&Cw20HookMsg::RepayStable {}).unwrap(),
    });

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "repay_stable"),
            attr("borrower", "addr0000"),
            attr("repay_amount", "100000"),
        ]
    );

    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "solid".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Burn {
                    amount: Uint128::from(99502u128),
                })
                .unwrap()
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "solid".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "collector".to_string(),
                    amount: Uint128::from(498u128),
                })
                .unwrap(),
            }))
        ]
    );

    //Loan amount and Total liability have decreased according to the repayment
    let res_loan = read_borrower_infos(deps.as_ref(), None, None)
        .unwrap()
        .get(0)
        .unwrap()
        .loan_amount;
    assert_eq!(res_loan, Uint256::from(402500u128));
    assert_eq!(
        read_state(deps.as_ref().storage).unwrap().total_liabilities,
        Decimal256::from_uint256(1400498u128)
    );

    // repay more then needed
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(500000u128),
        msg: to_binary(&Cw20HookMsg::RepayStable {}).unwrap(),
    });
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "repay_stable"),
            attr("borrower", "addr0000"),
            attr("repay_amount", "402500"),
        ]
    );

    //Loan amount and Total liability have decreased according to the repayment
    let res_loan = read_borrower_infos(deps.as_ref(), None, None)
        .unwrap()
        .get(0)
        .unwrap()
        .loan_amount;
    assert_eq!(res_loan, Uint256::zero());
    assert_eq!(
        read_state(deps.as_ref().storage).unwrap().total_liabilities,
        Decimal256::from_uint256(1000000u128)
    );

    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "solid".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "addr0000".to_string(),
                    amount: Uint128::from(97500u128),
                })
                .unwrap(),
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "solid".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Burn {
                    amount: Uint128::from(400498u128),
                })
                .unwrap()
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "solid".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "collector".to_string(),
                    amount: Uint128::from(2002u128),
                })
                .unwrap(),
            }))
        ]
    );

    // Borrow 0.005 fee repay with variable interest
    deps.querier.with_oracle_price(&[(
        &("solid".to_string(), "uusd".to_string()),
        &(
            Decimal256::from_str("1.02").unwrap(),
            env.block.time.seconds(),
            env.block.time.seconds(),
        ),
    )]);

    store_state(
        deps.as_mut().storage,
        &State {
            total_liabilities: Decimal256::from_uint256(1000000u128),
        },
    )
    .unwrap();

    let msg = ExecuteMsg::BorrowStable {
        borrow_amount: Uint256::from(500000u64),
        to: None,
    };

    env.block.height += 100;
    let info = mock_info("addr0000", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = mock_info("solid", &[]);

    // Repay part of the debt
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(100000u128),
        msg: to_binary(&Cw20HookMsg::RepayStable {}).unwrap(),
    });
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "repay_stable"),
            attr("borrower", "addr0000"),
            attr("repay_amount", "100000"),
        ]
    );

    //Loan amount and Total liability have decreased according to the repayment
    let res_loan = read_borrower_infos(deps.as_ref(), None, None)
        .unwrap()
        .get(0)
        .unwrap()
        .loan_amount;
    assert_eq!(res_loan, Uint256::from(402500u64));
    assert_eq!(
        read_state(deps.as_ref().storage).unwrap().total_liabilities,
        Decimal256::from_uint256(1400498u128)
    );

    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "solid".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Burn {
                    amount: Uint128::from(99502u128),
                })
                .unwrap()
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "solid".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "collector".to_string(),
                    amount: Uint128::from(498u128),
                })
                .unwrap(),
            }))
        ]
    );

    // Repay fee higher
    deps.querier.with_oracle_price(&[(
        &("solid".to_string(), "uusd".to_string()),
        &(
            Decimal256::from_str("1.03").unwrap(),
            env.block.time.seconds(),
            env.block.time.seconds(),
        ),
    )]);

    // Repay more then what is needed
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(500000u128),
        msg: to_binary(&Cw20HookMsg::RepayStable {}).unwrap(),
    });
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "repay_stable"),
            attr("borrower", "addr0000"),
            attr("repay_amount", "402500"),
        ]
    );

    //Loan amount and Total liability have decreased according to the repayment
    let res_loan = read_borrower_infos(deps.as_ref(), None, None)
        .unwrap()
        .get(0)
        .unwrap()
        .loan_amount;
    assert_eq!(res_loan, Uint256::zero());
    assert_eq!(
        read_state(deps.as_ref().storage).unwrap().total_liabilities,
        Decimal256::from_uint256(1000000u128)
    );

    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "solid".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "addr0000".to_string(),
                    amount: Uint128::from(97500u128),
                })
                .unwrap(),
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "solid".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Burn {
                    amount: Uint128::from(400498u128),
                })
                .unwrap()
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "solid".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "collector".to_string(),
                    amount: Uint128::from(2002u128),
                })
                .unwrap(),
            }))
        ]
    );
}

#[test]
fn repay_stable_from_liquidation() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner_addr: "owner".to_string(),
        stable_code_id: 123u64,
        base_borrow_fee: Decimal256::from_str("0.005").unwrap(),
        fee_increase_factor: Decimal256::from_str("2").unwrap(),
        fee_flash_mint: Decimal256::from_str("0.00025").unwrap(),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Register solid token contract
    let mut token_inst_res = MsgInstantiateContractResponse::new();
    token_inst_res.set_contract_address("solid".to_string());
    let reply_msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(token_inst_res.write_to_bytes().unwrap().into()),
        }),
    };
    let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    // Register overseer contract
    let msg = ExecuteMsg::RegisterContracts {
        overseer_contract: "overseer".to_string(),
        collector_contract: "collector".to_string(),
        liquidation_contract: "liquidation".to_string(),
        oracle_contract: "oracle".to_string(),
    };
    let env = mock_env();
    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    // mint fee = (1-0.99)/2 = 0.005 + 0.005 = 0.01
    deps.querier.with_oracle_price(&[(
        &("solid".to_string(), "uusd".to_string()),
        &(
            Decimal256::from_str("0.99").unwrap(),
            env.block.time.seconds(),
            env.block.time.seconds(),
        ),
    )]);
    deps.querier
        .with_borrow_limit(&[(&"addr0000".to_string(), &Uint256::from(1000000u64))]);

    store_state(
        deps.as_mut().storage,
        &State {
            total_liabilities: Decimal256::from_uint256(1000000u128),
        },
    )
    .unwrap();

    let info = mock_info("addr0000", &[]);
    // simulate borrow stable
    _borrow_stable(
        deps.as_mut(),
        env.clone(),
        info,
        Uint256::from(500000u64),
        Some(Addr::unchecked("addr0000".to_string())),
    )
    .unwrap();
    // repay fee = 0.005
    deps.querier.with_oracle_price(&[(
        &("solid".to_string(), "uusd".to_string()),
        &(
            Decimal256::from_str("1.01").unwrap(),
            env.block.time.seconds(),
            env.block.time.seconds(),
        ),
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(100000u128),
        msg: to_binary(&Cw20HookMsg::RepayStableFromLiquidation {
            borrower: "addr0000".to_string(),
        })
        .unwrap(),
    });

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg);
    // Unauthorized cw20 sender
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let info = mock_info("capa", &[]);

    // Unauthorized token sent
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "liquidation".to_string(),
        amount: Uint128::from(10u128),
        msg: to_binary(&Cw20HookMsg::RepayStableFromLiquidation {
            borrower: "addr0000".to_string(),
        })
        .unwrap(),
    });
    let res = execute(deps.as_mut(), env.clone(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }
    let info = mock_info("solid", &[]);

    // zero repay
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "liquidation".to_string(),
        amount: Uint128::from(0u128),
        msg: to_binary(&Cw20HookMsg::RepayStableFromLiquidation {
            borrower: "addr0000".to_string(),
        })
        .unwrap(),
    });

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    let _solid_denom = "Solid";
    match res {
        Err(ContractError::ZeroRepay(_solid_denom)) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }
    // partial repay
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "liquidation".to_string(),
        amount: Uint128::from(100000u128),
        msg: to_binary(&Cw20HookMsg::RepayStableFromLiquidation {
            borrower: "addr0000".to_string(),
        })
        .unwrap(),
    });

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "repay_stable"),
            attr("borrower", "addr0000"),
            attr("repay_amount", "100000"),
        ]
    );

    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "solid".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Burn {
                    amount: Uint128::from(99009u128),
                })
                .unwrap()
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "solid".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "collector".to_string(),
                    amount: Uint128::from(991u128),
                })
                .unwrap(),
            })),
        ]
    );

    deps.querier.with_oracle_price(&[(
        &("solid".to_string(), "uusd".to_string()),
        &(
            Decimal256::one(),
            env.block.time.seconds(),
            env.block.time.seconds(),
        ),
    )]);

    // repay more then needed
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "liquidation".to_string(),
        amount: Uint128::from(500000u128),
        msg: to_binary(&Cw20HookMsg::RepayStableFromLiquidation {
            borrower: "addr0000".to_string(),
        })
        .unwrap(),
    });

    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "repay_stable"),
            attr("borrower", "addr0000"),
            attr("repay_amount", "405000"),
        ]
    );

    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "solid".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "addr0000".to_string(),
                    amount: Uint128::from(95000u128),
                })
                .unwrap(),
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "solid".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Burn {
                    amount: Uint128::from(400991u128),
                })
                .unwrap()
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "solid".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "collector".to_string(),
                    amount: Uint128::from(4009u128),
                })
                .unwrap(),
            })),
        ]
    );

    let mut env = mock_env();
    let info = mock_info("addr0000", &[]);

    // simulate borrow stable again with 0.5% fee
    deps.querier.with_oracle_price(&[(
        &("solid".to_string(), "uusd".to_string()),
        &(
            Decimal256::one(),
            env.block.time.seconds(),
            env.block.time.seconds(),
        ),
    )]);

    _borrow_stable(
        deps.as_mut(),
        env.clone(),
        info,
        Uint256::from(500000u64),
        Some(Addr::unchecked("")),
    )
    .unwrap();

    // Update block height for interest
    env.block.height += 100;
    // update interest to 0.01
    deps.querier.with_oracle_price(&[(
        &("solid".to_string(), "uusd".to_string()),
        &(
            Decimal256::from_str("1.02").unwrap(),
            env.block.time.seconds(),
            env.block.time.seconds(),
        ),
    )]);

    let info = mock_info("solid", &[]);
    // repay exact amount + interest
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "liquidation".to_string(),
        amount: Uint128::from(502500u128),
        msg: to_binary(&Cw20HookMsg::RepayStableFromLiquidation {
            borrower: "addr0000".to_string(),
        })
        .unwrap(),
    });

    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "repay_stable"),
            attr("borrower", "addr0000"),
            attr("repay_amount", "502500"),
        ]
    );

    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "solid".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Burn {
                    amount: Uint128::from(500000u128),
                })
                .unwrap()
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "solid".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "collector".to_string(),
                    amount: Uint128::from(2500u128),
                })
                .unwrap(),
            })),
        ]
    );
}

#[test]
fn flash_mint() {
    let flash_mint_amount = Uint256::from_str("100000").unwrap();
    let flash_mint_fee = Decimal256::from_str("0.00025").unwrap();

    let flash_mint_fee_amount = flash_mint_amount * flash_mint_fee;

    let mut deps = mock_dependencies(&vec![]);

    let msg = InstantiateMsg {
        owner_addr: "owner".to_string(),
        stable_code_id: 123u64,
        base_borrow_fee: Decimal256::from_str("0.005").unwrap(),
        fee_increase_factor: Decimal256::from_str("2").unwrap(),
        fee_flash_mint: flash_mint_fee,
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Register solid token contract
    let mut token_inst_res = MsgInstantiateContractResponse::new();
    token_inst_res.set_contract_address("solid".to_string());
    let reply_msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(token_inst_res.write_to_bytes().unwrap().into()),
        }),
    };
    let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    // Register contracts
    let msg = ExecuteMsg::RegisterContracts {
        overseer_contract: "overseer".to_string(),
        collector_contract: "collector".to_string(),
        liquidation_contract: "liquidation".to_string(),
        oracle_contract: "oracle".to_string(),
    };
    let env = mock_env();
    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Flash mint request
    let info = mock_info("flash_minter", &[]);

    let msg = ExecuteMsg::FlashMint {
        amount: flash_mint_amount,
        msg_callback: to_binary("msg_callback").unwrap(),
    };

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Msgs that should be retrive
    let mut messages: Vec<SubMsg> = vec![];

    // Insert mint msg
    messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: String::from("solid"),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: String::from("flash_minter"),
            amount: flash_mint_amount.into(),
        })
        .unwrap(),
    })));

    // Insert callback msg
    messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: String::from("flash_minter"),
        funds: vec![],
        msg: to_binary("msg_callback").unwrap(),
    })));

    // Insert private flahs end msg
    messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        funds: vec![],
        msg: to_binary(&ExecuteMsg::PrivateFlashEnd {
            flash_minter: String::from("flash_minter"),
            burn_amount: flash_mint_amount.into(),
            fee_amount: flash_mint_fee_amount.into(),
        })
        .unwrap(),
    })));

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "flash_mint"),
            attr("flash_minter", "flash_minter"),
            attr("amount", flash_mint_amount),
            attr("fee_amount", flash_mint_amount * flash_mint_fee)
        ]
    );

    assert_eq!(res.messages, messages);

    // Call private flash end

    // Try to call from non env.contract.address
    // This has to fail
    let info = mock_info("flash_minter", &[]);

    let msg = ExecuteMsg::PrivateFlashEnd {
        flash_minter: String::from("random_address"),
        burn_amount: flash_mint_amount.into(),
        fee_amount: flash_mint_fee_amount.into(),
    };

    let res = execute(deps.as_mut(), env.clone(), info, msg);

    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // Call from env.contract.address
    let info = mock_info(env.contract.address.to_string().as_str(), &[]);

    let msg = ExecuteMsg::PrivateFlashEnd {
        flash_minter: String::from("flash_minter"),
        burn_amount: flash_mint_amount.into(),
        fee_amount: flash_mint_fee_amount.into(),
    };

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Msgs that should be retrive
    let mut messages: Vec<SubMsg> = vec![];

    // insert msg burn
    messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: String::from("solid"),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::BurnFrom {
            owner: String::from("flash_minter"),
            amount: flash_mint_amount.into(),
        })
        .unwrap(),
    })));

    // insert msg fee transfer to collector only if fee_amount > 0 (flash_mint_fee could be 0)
    if flash_mint_fee_amount > Uint256::zero() {
        messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: String::from("solid"),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: String::from("flash_minter"),
                recipient: String::from("collector"),
                amount: flash_mint_fee_amount.into(),
            })
            .unwrap(),
        })));
    }

    assert_eq!(res.messages, messages);
}

#[test]
fn migrate_contract() {
    let flash_mint_fee = Decimal256::zero();

    let mut deps = mock_dependencies(&vec![]);

    let msg = InstantiateMsg {
        owner_addr: "owner".to_string(),
        stable_code_id: 123u64,
        base_borrow_fee: Decimal256::from_str("0.005").unwrap(),
        fee_increase_factor: Decimal256::from_str("2").unwrap(),
        fee_flash_mint: flash_mint_fee,
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Register solid token contract
    let mut token_inst_res = MsgInstantiateContractResponse::new();
    token_inst_res.set_contract_address("solid".to_string());
    let reply_msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(token_inst_res.write_to_bytes().unwrap().into()),
        }),
    };
    let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    // Register contracts
    let msg = ExecuteMsg::RegisterContracts {
        overseer_contract: "overseer".to_string(),
        collector_contract: "collector".to_string(),
        liquidation_contract: "liquidation".to_string(),
        oracle_contract: "oracle".to_string(),
    };
    let env = mock_env();
    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Migrate
    let flash_mint_fee = Decimal256::from_str("0.00025").unwrap();

    let msg = MigrateMsg {
        owner_addr: Addr::unchecked("owner".to_string()),
        stable_contract: Addr::unchecked("solid".to_string()),
        overseer_contract: Addr::unchecked("overseer".to_string()),
        collector_contract: Addr::unchecked("collector".to_string()),
        liquidation_contract: Addr::unchecked("liquidation".to_string()),
        oracle_contract: Addr::unchecked("oracle".to_string()),
        base_borrow_fee: Decimal256::from_str("0.005").unwrap(),
        fee_increase_factor: Decimal256::from_str("2").unwrap(),
        flash_mint_fee: flash_mint_fee,
    };

    let _res = migrate(deps.as_mut(), env.clone(), msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_binary(&res).unwrap();

    assert_eq!(config_res.flash_mint_fee, flash_mint_fee)
}
