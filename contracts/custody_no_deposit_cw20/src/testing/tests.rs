use cosmwasm_bignumber::math::Uint256;
use cosmwasm_std::{from_binary, to_binary, Uint128};

use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use crate::testing::mock_querier::mock_dependencies;

use cosmwasm_std::testing::{mock_env, mock_info};
use cw20::Cw20ReceiveMsg;
use moneymarket::custody::{
    BorrowerResponse, ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg,
};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        collateral_token: "lunax".to_string(),
        overseer_contract: "overseer".to_string(),
        market_contract: "market".to_string(),
        liquidation_contract: "liquidation".to_string(),
        collector_contract: "collector".to_string(),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_binary(&query_res).unwrap();
    assert_eq!("owner".to_string(), config_res.owner);
    assert_eq!("lunax".to_string(), config_res.collateral_token);
    assert_eq!("overseer".to_string(), config_res.overseer_contract);
    assert_eq!("market".to_string(), config_res.market_contract);
    assert_eq!("liquidation".to_string(), config_res.liquidation_contract);
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        collateral_token: "lunax".to_string(),
        overseer_contract: "overseer".to_string(),
        market_contract: "market".to_string(),
        liquidation_contract: "liquidation".to_string(),
        collector_contract: "collector".to_string(),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::UpdateConfig {
        owner: Some("owner2".to_string()),
        liquidation_contract: Some("liquidation2".to_string()),
        collector_contract: Some("collector2".to_string()),
    };
    let info = mock_info("owner", &[]);
    execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_binary(&query_res).unwrap();
    assert_eq!("owner2".to_string(), config_res.owner);
    assert_eq!("lunax".to_string(), config_res.collateral_token);
    assert_eq!("overseer".to_string(), config_res.overseer_contract);
    assert_eq!("market".to_string(), config_res.market_contract);
    assert_eq!("liquidation2".to_string(), config_res.liquidation_contract);
    assert_eq!("collector2".to_string(), config_res.collector_contract);

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn deposit_collateral() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        collateral_token: "lunax".to_string(),
        overseer_contract: "overseer".to_string(),
        market_contract: "market".to_string(),
        liquidation_contract: "liquidation".to_string(),
        collector_contract: "collector".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::DepositCollateral {to: None}).unwrap(),
    });

    // failed; cannot directly execute receive message
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone());
    match res {
        Err(ContractError::DepositNotAllowed {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    //invalid message sent
    let msg2 = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary("invalid").unwrap(),
    });
    let res2 = execute(deps.as_mut(), mock_env(), info, msg2);
    match res2 {
        Err(ContractError::DepositNotAllowed {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let info = mock_info("lunax", &[]);
    let _ = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();

    let query_res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Borrower {
            address: "addr0000".to_string(),
        },
    )
    .unwrap();

    let borrower_res: BorrowerResponse = from_binary(&query_res).unwrap();
    assert_eq!(
        borrower_res,
        BorrowerResponse {
            borrower: "addr0000".to_string(),
            balance: Uint256::from(0u128),
            spendable: Uint256::from(0u128),
        }
    );

    // Deposit more
    let info = mock_info("lunax", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
   

    let query_res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Borrower {
            address: "addr0000".to_string(),
        },
    )
    .unwrap();
    let borrower_res: BorrowerResponse = from_binary(&query_res).unwrap();
    assert_eq!(
        borrower_res,
        BorrowerResponse {
            borrower: "addr0000".to_string(),
            balance: Uint256::from(0u128),
            spendable: Uint256::from(0u128),
        }
    );
}