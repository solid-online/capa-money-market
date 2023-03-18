use std::str::FromStr;

use crate::borrow::compute_borrow_fee;
use crate::state::{BorrowerInfo, Config};
use crate::testing::mock_querier::mock_dependencies;
use cosmwasm_bignumber::math::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{Addr, Coin, Uint128};

#[test]
fn proper_compute_borrower_interest() {
    let env = mock_env();
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(2000000u128),
    }]);

    let mock_config = Config {
        contract_addr: Addr::unchecked(MOCK_CONTRACT_ADDR),
        owner_addr: Addr::unchecked("owner"),
        stable_contract: Addr::unchecked("solid"),
        liquidation_contract: Addr::unchecked("liquidation"),
        collector_contract: Addr::unchecked("collector"),
        overseer_contract: Addr::unchecked("overseer"),
        oracle_contract: Addr::unchecked("oracle"),
        base_borrow_fee: Decimal256::from_str("0.005").unwrap(),
        fee_increase_factor: Decimal256::from_str("2").unwrap(),
        flash_mint_fee: Decimal256::from_str("0.00025").unwrap(),
    };
    // 1 solid price borrow_fee = 0
    deps.querier.with_oracle_price(&[(
        &("solid".to_string(), "uusd".to_string()),
        &(
            Decimal256::one(),
            env.block.time.seconds(),
            env.block.time.seconds(),
        ),
    )]);

    let mut liability1 = BorrowerInfo {
        loan_amount: Uint256::zero(),
        loan_amount_without_interest: Uint256::zero(),
    };
    let one_time_fee =
        compute_borrow_fee(deps.as_ref(), &env, &mock_config, liability1.loan_amount).unwrap();
    liability1.loan_amount += one_time_fee;
    let liability2 = BorrowerInfo {
        loan_amount: Uint256::zero(),
        loan_amount_without_interest: Uint256::zero(),
    };
    assert_eq!(liability1, liability2);

    // 0.98 solid price interest_rate = (1 - 0.98)/2 = 0.01 + 0.005 = 0.015
    deps.querier.with_oracle_price(&[(
        &("solid".to_string(), "uusd".to_string()),
        &(
            Decimal256::from_str("0.98").unwrap(),
            env.block.time.seconds(),
            env.block.time.seconds(),
        ),
    )]);

    let liability3 = BorrowerInfo {
        loan_amount: Uint256::from(800u128),
        loan_amount_without_interest: Uint256::zero(),
    };

    let liability4 = BorrowerInfo {
        loan_amount: Uint256::from(800u128),
        loan_amount_without_interest: Uint256::zero(),
    };
    assert_eq!(liability3, liability4);
}

#[test]
fn proper_compute_fee() {
    let env = mock_env();
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(2000000u128),
    }]);

    let _mock_config = Config {
        contract_addr: Addr::unchecked(MOCK_CONTRACT_ADDR),
        owner_addr: Addr::unchecked("owner"),
        stable_contract: Addr::unchecked("solid"),
        liquidation_contract: Addr::unchecked("liquidation"),
        collector_contract: Addr::unchecked("collector"),
        overseer_contract: Addr::unchecked("overseer"),
        oracle_contract: Addr::unchecked("oracle"),
        base_borrow_fee: Decimal256::from_str("0.005").unwrap(),
        fee_increase_factor: Decimal256::from_str("2").unwrap(),
        flash_mint_fee: Decimal256::from_str("0.00025").unwrap(),
    };
    // 1 solid price borrow_fee = 0 + 0.0025
    deps.querier.with_oracle_price(&[(
        &("solid".to_string(), "uusd".to_string()),
        &(
            Decimal256::one(),
            env.block.time.seconds(),
            env.block.time.seconds(),
        ),
    )]);

    let liability1 = BorrowerInfo {
        loan_amount: Uint256::zero(),
        loan_amount_without_interest: Uint256::zero(),
    };
    let liability2 = BorrowerInfo {
        loan_amount: Uint256::zero(),
        loan_amount_without_interest: Uint256::zero(),
    };
    assert_eq!(liability1, liability2);

    // 0.98 solid price repay_fee = 0
    deps.querier.with_oracle_price(&[(
        &("solid".to_string(), "uusd".to_string()),
        &(
            Decimal256::from_str("0.98").unwrap(),
            env.block.time.seconds(),
            env.block.time.seconds(),
        ),
    )]);

    let liability3 = BorrowerInfo {
        loan_amount: Uint256::from(800u128),
        loan_amount_without_interest: Uint256::zero(),
    };

    let liability4 = BorrowerInfo {
        loan_amount: Uint256::from(800u128),
        loan_amount_without_interest: Uint256::zero(),
    };
    assert_eq!(liability3, liability4);

    // 1.02 solid price interest_rate = (1.02 - 1)/2 = 0.01
    deps.querier.with_oracle_price(&[(
        &("solid".to_string(), "uusd".to_string()),
        &(
            Decimal256::from_str("1.02").unwrap(),
            env.block.time.seconds(),
            env.block.time.seconds(),
        ),
    )]);

    let liability5 = BorrowerInfo {
        loan_amount: Uint256::from(800u128),
        loan_amount_without_interest: Uint256::zero(),
    };

    let liability6 = BorrowerInfo {
        loan_amount: Uint256::from(800u128),
        loan_amount_without_interest: Uint256::zero(),
    };
    assert_eq!(liability5, liability6);
}
