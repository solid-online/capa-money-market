use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use cosmwasm_bignumber::math::Decimal256;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Addr, to_binary, OwnedDeps};
use moneymarket::oracle::{
    ConfigResponse, ExecuteMsg, FeederResponse, InstantiateMsg, PriceResponse, PriceSource,
    PricesResponse, PricesResponseElem, QueryMsg,
};
use std::str::FromStr;

use super::mock_querier::{oracle_mock_dependencies, QueryErisHub};

use rhaki_cw_mock_http_querier::mock::{create_http_mock, DefaultWasmMockQuerier, HttpWasmMockQuerier,};

const PULBLIC_NODE_URL: &str = "https://phoenix-lcd.terra.dev";

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        base_asset: "base0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("owner0000", value.owner.as_str());
    assert_eq!("base0000", &value.base_asset);
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        base_asset: "base0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // update owner
    let info = mock_info("owner0000", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some("owner0001".to_string()),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("owner0001", value.owner.as_str());
    assert_eq!("base0000", &value.base_asset);

    // Unauthorized err
    let info = mock_info("owner0000", &[]);
    let msg = ExecuteMsg::UpdateConfig { owner: None };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("Must return unauthorized error"),
    }
}

#[test]
fn update_feeder() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        base_asset: "base0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::UpdateFeeder {
        asset: "mAAPL".to_string(),
        feeder: Addr::unchecked("feeder0000".to_string()),
    };
    let info = mock_info("owner0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg);

    match res {
        Err(ContractError::AssetIsNotWhitelisted {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = ExecuteMsg::RegisterAsset {
        asset: "mAAPL".to_string(),
        source: PriceSource::Feeder {
            feeder: Addr::unchecked("feeder0000".to_string()),
            price: None,
            last_updated_time: None,
        },
    };

    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    let feeder_res: FeederResponse = from_binary(
        &query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Feeder {
                asset: "mAAPL".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        feeder_res,
        FeederResponse {
            asset: "mAAPL".to_string(),
            feeder: "feeder0000".to_string(),
        }
    );

    let msg = ExecuteMsg::UpdateFeeder {
        asset: "mAAPL".to_string(),
        feeder: Addr::unchecked("feeder0001".to_string()),
    };

    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let feeder_res: FeederResponse = from_binary(
        &query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Feeder {
                asset: "mAAPL".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        feeder_res,
        FeederResponse {
            asset: "mAAPL".to_string(),
            feeder: "feeder0001".to_string(),
        }
    );
}

#[test]
fn feed_price() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        base_asset: "base0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Register asset and feeder for mAAPL
    let msg = ExecuteMsg::RegisterAsset {
        asset: "mAAPL".to_string(),
        source: PriceSource::Feeder {
            feeder: Addr::unchecked("feeder0000".to_string()),
            price: None,
            last_updated_time: None,
        },
    };
    let info = mock_info("owner0000", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    // Register asset and feeder for mGOGL
    let msg = ExecuteMsg::RegisterAsset {
        asset: "mGOGL".to_string(),
        source: PriceSource::Feeder {
            feeder: Addr::unchecked("feeder0000".to_string()),
            price: None,
            last_updated_time: None,
        },
    };
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Feed prices
    let info = mock_info("feeder0000", &[]);
    let env = mock_env();
    let msg = ExecuteMsg::FeedPrice {
        prices: vec![
            ("mAAPL".to_string(), Decimal256::from_str("1.2").unwrap()),
            ("mGOGL".to_string(), Decimal256::from_str("2.2").unwrap()),
        ],
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Price {
            base: "mAAPL".to_string(),
            quote: "base0000".to_string(),
        },
    )
    .unwrap();
    let value: PriceResponse = from_binary(&res).unwrap();

    assert_eq!(
        value,
        PriceResponse {
            rate: Decimal256::from_str("1.2").unwrap(),
            last_updated_base: env.block.time.seconds(),
            last_updated_quote: env.block.time.seconds(),
        }
    );

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Price {
            base: "mGOGL".to_string(),
            quote: "mAAPL".to_string(),
        },
    )
    .unwrap();
    let value: PriceResponse = from_binary(&res).unwrap();

    assert_eq!(
        value,
        PriceResponse {
            rate: Decimal256::from_str("1.833333333333333333").unwrap(),
            last_updated_base: env.block.time.seconds(),
            last_updated_quote: env.block.time.seconds(),
        }
    );

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Prices {
            start_after: None,
            limit: None,
        },
    )
    .unwrap();
    let value: PricesResponse = from_binary(&res).unwrap();

    assert_eq!(
        value,
        PricesResponse {
            prices: vec![
                PricesResponseElem {
                    asset: "mAAPL".to_string(),
                    price: Decimal256::from_str("1.2").unwrap(),
                    last_updated_time: env.block.time.seconds(),
                },
                PricesResponseElem {
                    asset: "mGOGL".to_string(),
                    price: Decimal256::from_str("2.2").unwrap(),
                    last_updated_time: env.block.time.seconds(),
                }
            ],
        }
    );

    // Zero price feeder try
    let msg = ExecuteMsg::FeedPrice {
        prices: vec![
            ("mAAPL".to_string(), Decimal256::from_str("0").unwrap()),
            ("mGOGL".to_string(), Decimal256::from_str("2.2").unwrap()),
        ],
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::NotValidZeroPrice {}) => (),
        _ => panic!("Must return unauthorized error"),
    }

    // Unauthorized try
    let info = mock_info("addr0001", &[]);
    let msg = ExecuteMsg::FeedPrice {
        prices: vec![("mAAPL".to_string(), Decimal256::from_str("1.2").unwrap())],
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("Must return unauthorized error"),
    }
}

#[test]
fn lsd_price() {

    let mut deps = oracle_mock_dependencies( &vec![]);

    deps.querier.set_eris_querier("terra10788fkzah89xrdm27zkj5yvhj9x3494lxawzm5qq3vvxcqz2yzaqyd3enk".to_string(), Decimal256::from_str("1.1").unwrap());

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        base_asset: "base0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Register asset and feeder for uluna
    let msg = ExecuteMsg::RegisterAsset {
        asset: "uluna".to_string(),
        source: PriceSource::Feeder {
            feeder: Addr::unchecked("feeder0000".to_string()),
            price: None,
            last_updated_time: None,
        },
    };
    let info = mock_info("owner0000", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    // Register asset and feeder for usdc
    let msg = ExecuteMsg::RegisterAsset {
        asset: "usdc".to_string(),
        source: PriceSource::Feeder {
            feeder: Addr::unchecked("feeder0000".to_string()),
            price: None,
            last_updated_time: None,
        },
    };
    let info = mock_info("owner0000", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    // Feed prices
    let info = mock_info("feeder0000", &[]);
    let env = mock_env();
    let msg = ExecuteMsg::FeedPrice {
        prices: vec![
            ("uluna".to_string(), Decimal256::from_str("1.5").unwrap()),
            ("usdc".to_string(), Decimal256::from_str("1").unwrap()),
        ],
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Register asset and feeder for ampLuna
    let msg = ExecuteMsg::RegisterAsset {
        asset: "ampLuna".to_string(),
        source: PriceSource::LsdContractQuery {
            base_asset: "uluna".to_string(),
            contract: Addr::unchecked("terra10788fkzah89xrdm27zkj5yvhj9x3494lxawzm5qq3vvxcqz2yzaqyd3enk".to_string()),
            query_msg: to_binary(&QueryErisHub::State {}).unwrap(),
            path_key: vec!["state".to_string(), "exchange_rate".to_string()],
            is_inverted: false},
    };

    let info = mock_info("owner0000", &[]);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    println!("{:?}", res);

    // Get the price

    let msg = QueryMsg::Price { base: "ampLuna".to_string(), quote: "base0000".to_string() };

    let res:PriceResponse = from_binary(&query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();

    assert_eq!(res.rate, Decimal256::from_str("1.5").unwrap() * Decimal256::from_str("1.1").unwrap());

    // Query prices without pagination

    let msg = QueryMsg::Prices { start_after: None, limit: None };

    let res:PricesResponse = from_binary(&query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();

    assert_eq!(res, PricesResponse{prices: vec![
        PricesResponseElem{asset: "ampLuna".to_string(), price: Decimal256::from_str("1.5").unwrap() * Decimal256::from_str("1.1").unwrap(), last_updated_time: env.block.time.seconds()},
        PricesResponseElem{asset: "uluna".to_string(), price: Decimal256::from_str("1.5").unwrap(), last_updated_time: env.block.time.seconds()},
        PricesResponseElem{asset: "usdc".to_string(), price: Decimal256::from_str("1").unwrap(), last_updated_time: env.block.time.seconds()}
    ]});


    // Query prices with pagination

    let msg = QueryMsg::Prices { start_after: None, limit: Some(2_u32) };

    let res:PricesResponse = from_binary(&query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();

    assert_eq!(res, PricesResponse{prices: vec![
        PricesResponseElem{asset: "ampLuna".to_string(), price: Decimal256::from_str("1.5").unwrap() * Decimal256::from_str("1.1").unwrap(), last_updated_time: env.block.time.seconds()},
        PricesResponseElem{asset: "uluna".to_string(), price: Decimal256::from_str("1.5").unwrap(), last_updated_time: env.block.time.seconds()},
    ]});

    let msg = QueryMsg::Prices { start_after: Some("uluna".to_string()), limit: Some(2_u32) };

    let res:PricesResponse = from_binary(&query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();

    assert_eq!(res, PricesResponse{prices: vec![
        PricesResponseElem{asset: "usdc".to_string(), price: Decimal256::from_str("1").unwrap(), last_updated_time: env.block.time.seconds()}
    ]});

}

#[test]
fn lsd_price_http() {

    let mut deps: OwnedDeps<_, _, HttpWasmMockQuerier<DefaultWasmMockQuerier>>  = create_http_mock(None, PULBLIC_NODE_URL, None);

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        base_asset: "base0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Register asset and feeder for uluna
    let msg = ExecuteMsg::RegisterAsset {
        asset: "uluna".to_string(),
        source: PriceSource::Feeder {
            feeder: Addr::unchecked("feeder0000".to_string()),
            price: None,
            last_updated_time: None,
        },
    };
    let info = mock_info("owner0000", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    // Register asset and feeder for usdc
    let msg = ExecuteMsg::RegisterAsset {
        asset: "usdc".to_string(),
        source: PriceSource::Feeder {
            feeder: Addr::unchecked("feeder0000".to_string()),
            price: None,
            last_updated_time: None,
        },
    };
    let info = mock_info("owner0000", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    // Feed prices
    let info = mock_info("feeder0000", &[]);
    let env = mock_env();
    let msg = ExecuteMsg::FeedPrice {
        prices: vec![
            ("uluna".to_string(), Decimal256::from_str("1.5").unwrap()),
            ("usdc".to_string(), Decimal256::from_str("1").unwrap()),
        ],
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Register asset and feeder for ampLuna
    let msg = ExecuteMsg::RegisterAsset {
        asset: "ampLuna".to_string(),
        source: PriceSource::LsdContractQuery {
            base_asset: "uluna".to_string(),
            contract: Addr::unchecked("terra10788fkzah89xrdm27zkj5yvhj9x3494lxawzm5qq3vvxcqz2yzaqyd3enk".to_string()),
            query_msg: to_binary(&QueryErisHub::State {}).unwrap(),
            path_key: vec!["exchange_rate".to_string()],
            is_inverted: false},
    };

    let info = mock_info("owner0000", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    // Get the price

    let msg = QueryMsg::Price { base: "ampLuna".to_string(), quote: "base0000".to_string() };

    let res:PriceResponse = from_binary(&query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();

    println!("{:?}", res);

}
