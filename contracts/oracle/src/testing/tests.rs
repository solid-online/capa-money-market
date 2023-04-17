use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use cosmwasm_bignumber::math::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, Addr, OwnedDeps, Uint256 as StdUint256, Isqrt};
use moneymarket::oracle::{
    ConfigResponse, ExecuteMsg, FeederResponse, InstantiateMsg, PriceResponse, PricesResponse,
    PricesResponseElem, QueryMsg, RegisterPriceSource,
};
use std::str::FromStr;

use super::mock_querier::{oracle_mock_dependencies, AvaiableQueries, PoolStruct};

use rhaki_cw_mock_http_querier::mock::{
    create_http_mock, DefaultWasmMockQuerier, HttpWasmMockQuerier,
};

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
        Err(ContractError::AssetIsNotWhitelisted { .. }) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = ExecuteMsg::RegisterAsset {
        asset: "mAAPL".to_string(),
        source: RegisterPriceSource::Feeder {
            feeder: Addr::unchecked("feeder0000".to_string()),
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
        source: RegisterPriceSource::Feeder {
            feeder: Addr::unchecked("feeder0000".to_string()),
        },
    };
    let info = mock_info("owner0000", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    // Register asset and feeder for mGOGL
    let msg = ExecuteMsg::RegisterAsset {
        asset: "mGOGL".to_string(),
        source: RegisterPriceSource::Feeder {
            feeder: Addr::unchecked("feeder0000".to_string()),
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
    let mut deps = oracle_mock_dependencies(&[]);

    deps.querier.set_eris_querier(
        "terra10788fkzah89xrdm27zkj5yvhj9x3494lxawzm5qq3vvxcqz2yzaqyd3enk".to_string(),
        Decimal256::from_str("1.1").unwrap(),
    );

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        base_asset: "base0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Register asset and feeder for uluna
    let msg = ExecuteMsg::RegisterAsset {
        asset: "uluna".to_string(),
        source: RegisterPriceSource::Feeder {
            feeder: Addr::unchecked("feeder0000".to_string()),
        },
    };
    let info = mock_info("owner0000", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Register asset and feeder for usdc
    let msg = ExecuteMsg::RegisterAsset {
        asset: "usdc".to_string(),
        source: RegisterPriceSource::Feeder {
            feeder: Addr::unchecked("feeder0000".to_string()),
        },
    };
    let info = mock_info("owner0000", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Feed prices
    let info = mock_info("feeder0000", &[]);
    let env = mock_env();
    let msg = ExecuteMsg::FeedPrice {
        prices: vec![
            ("uluna".to_string(), Decimal256::from_str("1.5").unwrap()),
            ("usdc".to_string(), Decimal256::from_str("1").unwrap()),
        ],
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Register asset and feeder for ampLuna
    let msg = ExecuteMsg::RegisterAsset {
        asset: "ampLuna".to_string(),
        source: RegisterPriceSource::LsdContractQuery {
            base_asset: "uluna".to_string(),
            contract: Addr::unchecked(
                "terra10788fkzah89xrdm27zkj5yvhj9x3494lxawzm5qq3vvxcqz2yzaqyd3enk".to_string(),
            ),
            query_msg: to_binary(&AvaiableQueries::State {}).unwrap(),
            path_key: vec!["state".to_string(), "exchange_rate".to_string()],
            is_inverted: false,
        },
    };

    let info = mock_info("owner0000", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    println!("{:?}", res);

    // Get the price

    let msg = QueryMsg::Price {
        base: "ampLuna".to_string(),
        quote: "base0000".to_string(),
    };

    let res: PriceResponse = from_binary(&query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();

    assert_eq!(
        res.rate,
        Decimal256::from_str("1.5").unwrap() * Decimal256::from_str("1.1").unwrap()
    );

    // Query prices without pagination

    let msg = QueryMsg::Prices {
        start_after: None,
        limit: None,
    };

    let res: PricesResponse = from_binary(&query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();

    assert_eq!(
        res,
        PricesResponse {
            prices: vec![
                PricesResponseElem {
                    asset: "ampLuna".to_string(),
                    price: Decimal256::from_str("1.5").unwrap()
                        * Decimal256::from_str("1.1").unwrap(),
                    last_updated_time: env.block.time.seconds()
                },
                PricesResponseElem {
                    asset: "uluna".to_string(),
                    price: Decimal256::from_str("1.5").unwrap(),
                    last_updated_time: env.block.time.seconds()
                },
                PricesResponseElem {
                    asset: "usdc".to_string(),
                    price: Decimal256::from_str("1").unwrap(),
                    last_updated_time: env.block.time.seconds()
                }
            ]
        }
    );

    // Query prices with pagination

    let msg = QueryMsg::Prices {
        start_after: None,
        limit: Some(2_u32),
    };

    let res: PricesResponse = from_binary(&query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();

    assert_eq!(
        res,
        PricesResponse {
            prices: vec![
                PricesResponseElem {
                    asset: "ampLuna".to_string(),
                    price: Decimal256::from_str("1.5").unwrap()
                        * Decimal256::from_str("1.1").unwrap(),
                    last_updated_time: env.block.time.seconds()
                },
                PricesResponseElem {
                    asset: "uluna".to_string(),
                    price: Decimal256::from_str("1.5").unwrap(),
                    last_updated_time: env.block.time.seconds()
                },
            ]
        }
    );

    let msg = QueryMsg::Prices {
        start_after: Some("uluna".to_string()),
        limit: Some(2_u32),
    };

    let res: PricesResponse = from_binary(&query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();

    assert_eq!(
        res,
        PricesResponse {
            prices: vec![PricesResponseElem {
                asset: "usdc".to_string(),
                price: Decimal256::from_str("1").unwrap(),
                last_updated_time: env.block.time.seconds()
            }]
        }
    );
}

#[test]
fn lsd_price_http() {
    let mut deps: OwnedDeps<_, _, HttpWasmMockQuerier<DefaultWasmMockQuerier>> =
        create_http_mock(None, PULBLIC_NODE_URL, None);

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        base_asset: "base0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Register asset and feeder for uluna
    let msg = ExecuteMsg::RegisterAsset {
        asset: "uluna".to_string(),
        source: RegisterPriceSource::Feeder {
            feeder: Addr::unchecked("feeder0000".to_string()),
        },
    };
    let info = mock_info("owner0000", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Register asset and feeder for usdc
    let msg = ExecuteMsg::RegisterAsset {
        asset: "usdc".to_string(),
        source: RegisterPriceSource::Feeder {
            feeder: Addr::unchecked("feeder0000".to_string()),
        },
    };
    let info = mock_info("owner0000", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Feed prices
    let info = mock_info("feeder0000", &[]);
    let env = mock_env();
    let msg = ExecuteMsg::FeedPrice {
        prices: vec![
            ("uluna".to_string(), Decimal256::from_str("1.5").unwrap()),
            ("usdc".to_string(), Decimal256::from_str("1").unwrap()),
        ],
    };
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();

    // Register asset and feeder for ampLuna
    let msg = ExecuteMsg::RegisterAsset {
        asset: "ampLuna".to_string(),
        source: RegisterPriceSource::LsdContractQuery {
            base_asset: "uluna".to_string(),
            contract: Addr::unchecked(
                "terra10788fkzah89xrdm27zkj5yvhj9x3494lxawzm5qq3vvxcqz2yzaqyd3enk".to_string(),
            ),
            query_msg: to_binary(&AvaiableQueries::State {}).unwrap(),
            path_key: vec!["exchange_rate".to_string()],
            is_inverted: false,
        },
    };

    let info = mock_info("owner0000", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Get the price

    let msg = QueryMsg::Price {
        base: "ampLuna".to_string(),
        quote: "base0000".to_string(),
    };

    let res: PriceResponse = from_binary(&query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();

    println!("{:?}", res);
}

#[test]
fn astroport_lp_autocompound() {

    let amount_usdc_in_pool: Uint256 = Uint256::from(50_000_u128);
    let amount_uluna_in_pool: Uint256 = Uint256::from(25_000_u128);

    let price_usdc = Decimal256::from_str("1").unwrap();
    let price_luna = Decimal256::from_str("2").unwrap();

    let pool_usdc_uluna =  Addr::unchecked("pool_usdc_uluna".to_string());

    let lp_usdc_luna =  Addr::unchecked("lp_usdc_uluna".to_string());
    let clp_usdc_luna =  Addr::unchecked("clp_usdc_uluna".to_string());

    let vault = Addr::unchecked("vault".to_string());

    let supply_lp_usdc_uluna: Uint256 = Uint256::from(100_000_u128);
    let staked_lp_usdc_uluna: Uint256 = Uint256::from(20_000_u128);
    let supply_clp_usdc_uluna: Uint256 = Uint256::from(20_000_u128);

    let mut deps = oracle_mock_dependencies(&[]);

    deps.querier.set_token_supply(lp_usdc_luna.clone(), supply_lp_usdc_uluna);
    deps.querier.set_token_supply(clp_usdc_luna.clone(), supply_clp_usdc_uluna);

    deps.querier.set_generator_lp_stake(vault.clone(), lp_usdc_luna.clone(), staked_lp_usdc_uluna);

    deps.querier.set_pool_info(pool_usdc_uluna.clone(), PoolStruct{
        assets: vec![("usdc".to_string(), amount_usdc_in_pool), ("uluna".to_string(), amount_uluna_in_pool)],
        lp: lp_usdc_luna,
    });


    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        base_asset: "base0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Register asset and feeder for uluna
    let msg = ExecuteMsg::RegisterAsset {
        asset: "uluna".to_string(),
        source: RegisterPriceSource::Feeder {
            feeder: Addr::unchecked("feeder0000".to_string()),
        },
    };

    let info = mock_info("owner0000", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Register asset and feeder for usdc
    let msg = ExecuteMsg::RegisterAsset {
        asset: "usdc".to_string(),
        source: RegisterPriceSource::Feeder {
            feeder: Addr::unchecked("feeder0000".to_string()),
        },
    };

    let info = mock_info("owner0000", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Feed prices
    let env = mock_env();
    let msg = ExecuteMsg::FeedPrice {
        prices: vec![
            ("uluna".to_string(), price_luna.clone()),
            ("usdc".to_string(), price_usdc.clone()),
        ],
    };

    let info = mock_info("feeder0000", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Register clp
    let msg = ExecuteMsg::RegisterAsset {
        asset: clp_usdc_luna.to_string(),
        source: RegisterPriceSource::AstroportLpAutocompound { 
            vault_contract: vault,
            generator_contract: Addr::unchecked("generator".to_string()),
            pool_contract: pool_usdc_uluna }
        ,
    };

    let info = mock_info("owner0000", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let msg = QueryMsg::Price {
        base: clp_usdc_luna.to_string(),
        quote: "base0000".to_string(),
    };

    let res: PriceResponse = from_binary(&query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();

    let mul = price_luna * amount_uluna_in_pool * price_usdc * amount_usdc_in_pool;

    let pool_value = StdUint256::from(2u8) * StdUint256::from_u128(mul.into()).isqrt();

    let clp_price = 
    Decimal256::from_ratio(
        Decimal256::from_ratio(staked_lp_usdc_uluna, supply_lp_usdc_uluna) 
        * Uint256::from_str(pool_value.to_string().as_str()).unwrap(),
        supply_clp_usdc_uluna)
    ;

    assert_eq!(res, PriceResponse{
        rate:clp_price,
        last_updated_base: env.block.time.seconds(),
        last_updated_quote: env.block.time.seconds()
    });

    // assert_eq!(res, PriceResponse{
    //     rate:Decimal256::from_str(&"2".to_string()).unwrap(),
    //     last_updated_base: env.block.time.seconds(),
    //     last_updated_quote: env.block.time.seconds()
    // });

    println!("{:?}", res)
}

#[test]
fn astroport_lp_autocompound_http() {

    let usdc = "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string();
    let uluna = "uluna".to_string();

    let price_usdc = Decimal256::from_str("1").unwrap();
    let price_luna = Decimal256::from_str("1.36").unwrap();

    // ERIS PROTOCOL FARM USDC-LUNA

    let pool_usdc_uluna =  Addr::unchecked("terra1fd68ah02gr2y8ze7tm9te7m70zlmc7vjyyhs6xlhsdmqqcjud4dql4wpxr".to_string());

    let clp_usdc_luna =  Addr::unchecked("terra1as76h247wvey3aqmw22mlkq8g6vj8zj7qw4wywwn388s2mjt0rtqpp570z".to_string());

    let vault = Addr::unchecked("terra1xskgvsew6u6nmfwv2mc58m4hscr77xw884x65fuxup8ewvvvuyysr5k3lj".to_string());

    let generator = Addr::unchecked("terra1m42utlz6uvnlzn82f58pfkkuxw8j9vf24hf00t54qfn4k23fhj3q70vqd0".to_string());


    let mut deps: OwnedDeps<_, _, HttpWasmMockQuerier<DefaultWasmMockQuerier>> =
    create_http_mock(None, PULBLIC_NODE_URL, None);


    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        base_asset: "base0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Register asset and feeder for uluna
    let msg = ExecuteMsg::RegisterAsset {
        asset: uluna.to_string(),
        source: RegisterPriceSource::Feeder {
            feeder: Addr::unchecked("feeder0000".to_string()),
        },
    };

    let info = mock_info("owner0000", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Register asset and feeder for usdc
    let msg = ExecuteMsg::RegisterAsset {
        asset: usdc.to_string(),
        source: RegisterPriceSource::Feeder {
            feeder: Addr::unchecked("feeder0000".to_string()),
        },
    };

    let info = mock_info("owner0000", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Feed prices
    let env = mock_env();
    let msg = ExecuteMsg::FeedPrice {
        prices: vec![
            (usdc, price_luna.clone()),
            (uluna, price_usdc.clone()),
        ],
    };

    let info = mock_info("feeder0000", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Register clp
    let msg = ExecuteMsg::RegisterAsset {
        asset: clp_usdc_luna.to_string(),
        source: RegisterPriceSource::AstroportLpAutocompound { 
            vault_contract: vault,
            generator_contract: generator,
            pool_contract: pool_usdc_uluna }
        ,
    };

    let info = mock_info("owner0000", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let msg = QueryMsg::Price {
        base: clp_usdc_luna.to_string(),
        quote: "base0000".to_string(),
    };

    let res: PriceResponse = from_binary(&query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();

    println!("{:?}", res)


}