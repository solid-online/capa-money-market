use crate::collateral::lock_collateral as _lock_collateral;
use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use crate::testing::mock_querier::mock_dependencies;

use cosmwasm_bignumber::math::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{attr, from_binary, to_binary, CosmosMsg, SubMsg, WasmMsg};

use moneymarket::custody::ExecuteMsg as CustodyExecuteMsg;

use moneymarket::overseer::{
    AllCollateralsResponse, BorrowLimitResponse, CollateralsResponse, ConfigResponse, ExecuteMsg,
    InstantiateMsg, QueryMsg, WhitelistResponse, WhitelistResponseElem,
};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner_addr: "owner".to_string(),
        oracle_contract: "oracle".to_string(),
        market_contract: "market".to_string(),
        liquidation_contract: "liquidation".to_string(),
        collector_contract: "collector".to_string(),
        stable_contract: "uusd".to_string(),
        price_timeframe: 60u64,
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_binary(&query_res).unwrap();
    assert_eq!(
        config_res,
        ConfigResponse {
            owner_addr: "owner".to_string(),
            oracle_contract: "oracle".to_string(),
            market_contract: "market".to_string(),
            liquidation_contract: "liquidation".to_string(),
            collector_contract: "collector".to_string(),
            stable_contract: "uusd".to_string(),
            price_timeframe: 60u64,
        }
    );
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);

    let info = mock_info("addr0000", &[]);
    let msg = InstantiateMsg {
        owner_addr: "owner".to_string(),
        oracle_contract: "oracle".to_string(),
        market_contract: "market".to_string(),
        liquidation_contract: "liquidation".to_string(),
        collector_contract: "collector".to_string(),
        stable_contract: "uusd".to_string(),
        price_timeframe: 60u64,
    };

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // update owner
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        owner_addr: Some("owner1".to_string()),
        oracle_contract: None,
        liquidation_contract: None,
        price_timeframe: None,
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
        oracle_contract: Some("oracle1".to_string()),
        liquidation_contract: Some("liquidation1".to_string()),
        price_timeframe: Some(120u64),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("owner1".to_string(), config_res.owner_addr);
    assert_eq!("oracle1".to_string(), config_res.oracle_contract);
    assert_eq!("liquidation1".to_string(), config_res.liquidation_contract);
    assert_eq!(120u64, config_res.price_timeframe);

    // Unauthorized err
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        owner_addr: None,
        oracle_contract: None,
        liquidation_contract: None,
        price_timeframe: None,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("Must return unauthorized error"),
    }
}

#[test]
fn whitelist() {
    let mut deps = mock_dependencies(&[]);

    let info = mock_info("addr0000", &[]);
    let msg = InstantiateMsg {
        owner_addr: "owner".to_string(),
        oracle_contract: "oracle".to_string(),
        market_contract: "market".to_string(),
        liquidation_contract: "liquidation".to_string(),
        collector_contract: "collector".to_string(),
        stable_contract: "uusd".to_string(),
        price_timeframe: 60u64,
    };

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::Whitelist {
        name: "bluna".to_string(),
        symbol: "bluna".to_string(),
        collateral_token: "bluna".to_string(),
        custody_contract: "custody".to_string(),
        max_ltv: Decimal256::from_ratio(100, 1),
    };

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::InvalidMaxLtv {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    };

    let msg = ExecuteMsg::Whitelist {
        name: "bluna".to_string(),
        symbol: "bluna".to_string(),
        collateral_token: "bluna".to_string(),
        custody_contract: "custody".to_string(),
        max_ltv: Decimal256::zero(),
    };

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::InvalidMaxLtv {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    };

    let msg = ExecuteMsg::Whitelist {
        name: "bluna".to_string(),
        symbol: "bluna".to_string(),
        collateral_token: "bluna".to_string(),
        custody_contract: "custody".to_string(),
        max_ltv: Decimal256::percent(60),
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    };
    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "register_whitelist"),
            attr("name", "bluna"),
            attr("symbol", "bluna"),
            attr("collateral_token", "bluna"),
            attr("custody_contract", "custody"),
            attr("LTV", "0.6"),
        ]
    );

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Whitelist {
            collateral_token: Some("bluna".to_string()),
            start_after: None,
            limit: None,
        },
    )
    .unwrap();
    let whitelist_res: WhitelistResponse = from_binary(&res).unwrap();
    assert_eq!(
        whitelist_res,
        WhitelistResponse {
            elems: vec![WhitelistResponseElem {
                name: "bluna".to_string(),
                symbol: "bluna".to_string(),
                collateral_token: "bluna".to_string(),
                custody_contract: "custody".to_string(),
                max_ltv: Decimal256::percent(60),
            }]
        }
    );

    //Attempting to whitelist already whitelisted collaterals
    let msg = ExecuteMsg::Whitelist {
        name: "bluna".to_string(),
        symbol: "bluna".to_string(),
        collateral_token: "bluna".to_string(),
        custody_contract: "custody".to_string(),
        max_ltv: Decimal256::percent(60),
    };

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    match res {
        ContractError::TokenAlreadyRegistered {} => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = ExecuteMsg::UpdateWhitelist {
        collateral_token: "bluna".to_string(),
        custody_contract: Some("custody2".to_string()),
        max_ltv: Some(Decimal256::from_ratio(105, 1)),
    };

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::InvalidMaxLtv {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    };

    let msg = ExecuteMsg::UpdateWhitelist {
        collateral_token: "bluna".to_string(),
        custody_contract: Some("custody2".to_string()),
        max_ltv: Some(Decimal256::zero()),
    };

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::InvalidMaxLtv {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    };

    let msg = ExecuteMsg::UpdateWhitelist {
        collateral_token: "bluna".to_string(),
        custody_contract: Some("custody2".to_string()),
        max_ltv: Some(Decimal256::percent(30)),
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    };

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "update_whitelist"),
            attr("collateral_token", "bluna"),
            attr("custody_contract", "custody2"),
            attr("LTV", "0.3"),
        ]
    );

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Whitelist {
            collateral_token: Some("bluna".to_string()),
            start_after: None,
            limit: None,
        },
    )
    .unwrap();
    let whitelist_res: WhitelistResponse = from_binary(&res).unwrap();
    assert_eq!(
        whitelist_res,
        WhitelistResponse {
            elems: vec![WhitelistResponseElem {
                name: "bluna".to_string(),
                symbol: "bluna".to_string(),
                collateral_token: "bluna".to_string(),
                custody_contract: "custody2".to_string(),
                max_ltv: Decimal256::percent(30),
            }]
        }
    );
}

#[test]
fn lock_collateral() {
    let mut deps = mock_dependencies(&[]);

    let info = mock_info("owner", &[]);
    let msg = InstantiateMsg {
        owner_addr: "owner".to_string(),
        oracle_contract: "oracle".to_string(),
        market_contract: "market".to_string(),
        liquidation_contract: "liquidation".to_string(),
        collector_contract: "collector".to_string(),
        stable_contract: "uusd".to_string(),
        price_timeframe: 60u64,
    };

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let batom_collat_token = "batom".to_string();

    let bluna_collat_token = "bluna".to_string();

    // store whitelist elems
    let msg = ExecuteMsg::Whitelist {
        name: "bluna".to_string(),
        symbol: "bluna".to_string(),
        collateral_token: bluna_collat_token.clone(),
        custody_contract: "custody_bluna".to_string(),
        max_ltv: Decimal256::percent(60),
    };

    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg);

    let msg = ExecuteMsg::Whitelist {
        name: "batom".to_string(),
        symbol: "batom".to_string(),
        collateral_token: batom_collat_token.clone(),
        custody_contract: "custody_batom".to_string(),
        max_ltv: Decimal256::percent(60),
    };

    let _res = execute(deps.as_mut(), mock_env(), info, msg);

    let msg = ExecuteMsg::LockCollateral {
        collaterals: vec![
            (bluna_collat_token.clone(), Uint256::from(1000000u64)),
            (batom_collat_token.clone(), Uint256::from(10000000u64)),
        ],
    };
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "custody_bluna".to_string(),
                funds: vec![],
                msg: to_binary(&CustodyExecuteMsg::LockCollateral {
                    borrower: "addr0000".to_string(),
                    amount: Uint256::from(1000000u64),
                })
                .unwrap(),
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "custody_batom".to_string(),
                funds: vec![],
                msg: to_binary(&CustodyExecuteMsg::LockCollateral {
                    borrower: "addr0000".to_string(),
                    amount: Uint256::from(10000000u64),
                })
                .unwrap(),
            }))
        ]
    );

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "lock_collateral"),
            attr("borrower", "addr0000"),
            attr(
                "collaterals",
                format!(
                    "1000000{},10000000{}",
                    bluna_collat_token, batom_collat_token
                )
            ),
        ]
    );

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Collaterals {
            borrower: "addr0000".to_string(),
        },
    )
    .unwrap();
    let collaterals_res: CollateralsResponse = from_binary(&res).unwrap();
    assert_eq!(
        collaterals_res,
        CollateralsResponse {
            borrower: "addr0000".to_string(),
            collaterals: vec![
                (batom_collat_token.clone(), Uint256::from(10000000u64)),
                (bluna_collat_token.clone(), Uint256::from(1000000u64)),
            ]
        }
    );

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::AllCollaterals {
            start_after: None,
            limit: None,
        },
    )
    .unwrap();
    let all_collaterals_res: AllCollateralsResponse = from_binary(&res).unwrap();
    assert_eq!(
        all_collaterals_res,
        AllCollateralsResponse {
            all_collaterals: vec![CollateralsResponse {
                borrower: "addr0000".to_string(),
                collaterals: vec![
                    (batom_collat_token, Uint256::from(10000000u64)),
                    (bluna_collat_token, Uint256::from(1000000u64)),
                ]
            }]
        }
    );
}

#[test]
fn unlock_collateral() {
    let mut deps = mock_dependencies(&[]);

    let info = mock_info("owner", &[]);
    let env = mock_env();
    let msg = InstantiateMsg {
        owner_addr: "owner".to_string(),
        oracle_contract: "oracle".to_string(),
        market_contract: "market".to_string(),
        liquidation_contract: "liquidation".to_string(),
        collector_contract: "collector".to_string(),
        stable_contract: "uusd".to_string(),
        price_timeframe: 60u64,
    };

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // store whitelist elems
    let msg = ExecuteMsg::Whitelist {
        name: "bluna".to_string(),
        symbol: "bluna".to_string(),
        collateral_token: "bluna".to_string(),
        custody_contract: "custody_bluna".to_string(),
        max_ltv: Decimal256::percent(60),
    };

    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

    let msg = ExecuteMsg::Whitelist {
        name: "batom".to_string(),
        symbol: "batom".to_string(),
        collateral_token: "batom".to_string(),
        custody_contract: "custody_batom".to_string(),
        max_ltv: Decimal256::percent(60),
    };

    let _res = execute(deps.as_mut(), env.clone(), info, msg);

    let collaterals = vec![
        ("bluna".to_string(), Uint256::from(1000000u64)),
        ("batom".to_string(), Uint256::from(10000000u64)),
    ];

    let info = mock_info("addr0000", &[]);

    // simulate lock collateral
    _lock_collateral(deps.as_mut(), info.clone(), collaterals).unwrap();

    // Failed to unlock more than locked amount
    let msg = ExecuteMsg::UnlockCollateral {
        collaterals: vec![
            ("bluna".to_string(), Uint256::from(1000001u64)),
            ("batom".to_string(), Uint256::from(10000001u64)),
        ],
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    match res {
        Err(ContractError::UnlockExceedsLocked {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    deps.querier.with_oracle_price(&[
        (
            &("bluna".to_string(), "uusd".to_string()),
            &(
                Decimal256::from_ratio(1000u64, 1u64),
                env.block.time.seconds(),
                env.block.time.seconds(),
            ),
        ),
        (
            &("batom".to_string(), "uusd".to_string()),
            &(
                Decimal256::from_ratio(2000u64, 1u64),
                env.block.time.seconds(),
                env.block.time.seconds(),
            ),
        ),
    ]);

    // borrow_limit = 1000 * 1000000 * 0.6 + 2000 * 10000000 * 0.6
    // = 12,600,000,000 uusd
    deps.querier
        .with_loan_amount(&[(&"addr0000".to_string(), &Uint256::from(12600000000u64))]);

    // cannot unlock any tokens
    // Failed to unlock more than locked amount
    let msg = ExecuteMsg::UnlockCollateral {
        collaterals: vec![("bluna".to_string(), Uint256::one())],
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    match res {
        Err(ContractError::UnlockTooLarge(12599999400)) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = ExecuteMsg::UnlockCollateral {
        collaterals: vec![("batom".to_string(), Uint256::one())],
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    match res {
        Err(ContractError::UnlockTooLarge(12599998800)) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // borrow_limit = 1000 * 1000000 * 0.6 + 2000 * 10000000 * 0.6
    // = 12,600,000,000 uusd
    deps.querier
        .with_loan_amount(&[(&"addr0000".to_string(), &Uint256::from(12599999400u64))]);
    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::BorrowLimit {
            borrower: "addr0000".to_string(),
            block_time: None,
        },
    )
    .unwrap();
    let borrow_limit_res: BorrowLimitResponse = from_binary(&res).unwrap();
    assert_eq!(borrow_limit_res.borrow_limit, Uint256::from(12600000000u64),);

    // Cannot unlock 2bluna
    let msg = ExecuteMsg::UnlockCollateral {
        collaterals: vec![("bluna".to_string(), Uint256::from(2u64))],
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    match res {
        Err(ContractError::UnlockTooLarge(12599998800)) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // Can unlock 1bluna
    let msg = ExecuteMsg::UnlockCollateral {
        collaterals: vec![("bluna".to_string(), Uint256::one())],
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "custody_bluna".to_string(),
            funds: vec![],
            msg: to_binary(&CustodyExecuteMsg::UnlockCollateral {
                borrower: "addr0000".to_string(),
                amount: Uint256::one(),
            })
            .unwrap(),
        }))]
    );

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "unlock_collateral"),
            attr("borrower", "addr0000"),
            attr("collaterals", "1bluna"),
        ]
    );

    //testing for unlocking more collaterals
    deps.querier
        .with_loan_amount(&[(&"addr0000".to_string(), &Uint256::from(125999900u128))]);

    let msg = ExecuteMsg::UnlockCollateral {
        collaterals: vec![
            ("bluna".to_string(), Uint256::from(1u128)),
            ("batom".to_string(), Uint256::from(1u128)),
        ],
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "custody_bluna".to_string(),
                funds: vec![],
                msg: to_binary(&CustodyExecuteMsg::UnlockCollateral {
                    borrower: "addr0000".to_string(),
                    amount: Uint256::from(1u128),
                })
                .unwrap(),
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "custody_batom".to_string(),
                funds: vec![],
                msg: to_binary(&CustodyExecuteMsg::UnlockCollateral {
                    borrower: "addr0000".to_string(),
                    amount: Uint256::from(1u128),
                })
                .unwrap(),
            }))
        ]
    );
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "unlock_collateral"),
            attr("borrower", "addr0000"),
            attr("collaterals", "1bluna,1batom"),
        ]
    );
}

#[test]
fn liquidate_collateral() {
    let mut deps = mock_dependencies(&[]);
    deps.querier
        .with_liquidation_percent(&[(&"liquidation".to_string(), &Decimal256::percent(1))]);

    let info = mock_info("owner", &[]);
    let env = mock_env();
    let msg = InstantiateMsg {
        owner_addr: "owner".to_string(),
        oracle_contract: "oracle".to_string(),
        market_contract: "market".to_string(),
        liquidation_contract: "liquidation".to_string(),
        collector_contract: "collector".to_string(),
        stable_contract: "uusd".to_string(),
        price_timeframe: 60u64,
    };

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let batom_collat_token = "batom".to_string();

    let bluna_collat_token = "bluna".to_string();

    // store whitelist elems
    let msg = ExecuteMsg::Whitelist {
        name: "bluna".to_string(),
        symbol: "bluna".to_string(),
        collateral_token: bluna_collat_token.clone(),
        custody_contract: "custody_bluna".to_string(),
        max_ltv: Decimal256::percent(60),
    };

    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

    let msg = ExecuteMsg::Whitelist {
        name: "batom".to_string(),
        symbol: "batom".to_string(),
        collateral_token: batom_collat_token.clone(),
        custody_contract: "custody_batom".to_string(),
        max_ltv: Decimal256::percent(60),
    };

    let _res = execute(deps.as_mut(), env.clone(), info, msg);

    let collaterals = vec![
        (bluna_collat_token.clone(), Uint256::from(1000000u64)),
        (batom_collat_token.clone(), Uint256::from(10000000u64)),
    ];

    let info = mock_info("addr0000", &[]);

    // simulate lock collateral
    _lock_collateral(deps.as_mut(), info, collaterals).unwrap();

    deps.querier.with_oracle_price(&[
        (
            &(bluna_collat_token.clone(), "uusd".to_string()),
            &(
                Decimal256::from_ratio(1000u64, 1u64),
                env.block.time.seconds(),
                env.block.time.seconds(),
            ),
        ),
        (
            &(batom_collat_token.clone(), "uusd".to_string()),
            &(
                Decimal256::from_ratio(2000u64, 1u64),
                env.block.time.seconds(),
                env.block.time.seconds(),
            ),
        ),
    ]);

    // borrow_limit = 1000 * 1000000 * 0.6 + 2000 * 10000000 * 0.6
    // = 12,600,000,000 uusd
    deps.querier
        .with_loan_amount(&[(&"addr0000".to_string(), &Uint256::from(12600000000u64))]);

    let msg = ExecuteMsg::LiquidateCollateral {
        borrower: "addr0000".to_string(),
    };
    let info = mock_info("addr0001", &[]);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    match res {
        Err(ContractError::CannotLiquidateSafeLoan {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    deps.querier
        .with_loan_amount(&[(&"addr0000".to_string(), &Uint256::from(12600000001u64))]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "custody_batom".to_string(),
                funds: vec![],
                msg: to_binary(&CustodyExecuteMsg::LiquidateCollateral {
                    liquidator: "addr0001".to_string(),
                    borrower: "addr0000".to_string(),
                    amount: Uint256::from(100000u64),
                })
                .unwrap(),
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "custody_bluna".to_string(),
                funds: vec![],
                msg: to_binary(&CustodyExecuteMsg::LiquidateCollateral {
                    liquidator: "addr0001".to_string(),
                    borrower: "addr0000".to_string(),
                    amount: Uint256::from(10000u64),
                })
                .unwrap(),
            })),
        ]
    );

    let res = query(
        deps.as_ref(),
        env,
        QueryMsg::Collaterals {
            borrower: "addr0000".to_string(),
        },
    )
    .unwrap();
    let collaterals_res: CollateralsResponse = from_binary(&res).unwrap();
    assert_eq!(
        collaterals_res,
        CollateralsResponse {
            borrower: "addr0000".to_string(),
            collaterals: vec![
                (batom_collat_token, Uint256::from(9900000u64)),
                (bluna_collat_token, Uint256::from(990000u64)),
            ]
        }
    );
}
