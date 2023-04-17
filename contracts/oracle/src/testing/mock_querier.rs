use std::{marker::PhantomData, collections::HashMap};

use astroport::asset::{AssetInfo, PairInfo, Asset};
use astroport::factory::PairType;
use astroport::pair::{PoolResponse, QueryMsg as PairQueryMsg};
use cosmwasm_bignumber::math::{Decimal256, Uint256};
use cosmwasm_std::Uint128;
use cosmwasm_std::{
    from_binary, from_slice,
    testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR},
    to_binary, Coin, ContractResult, OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError,
    SystemResult, WasmQuery, Addr,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terra_cosmwasm::TerraQueryWrapper;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AvaiableQueries {
    State {},
    Pair {},
    Pool {},
    TokenInfo{},
    Deposit {lp_token:String, user:String}
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TokenInfoResponse {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryErisHubResponse {
    State { exchange_rate: Decimal256 },
}

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn oracle_mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: PhantomData::default(),
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    eris_querier: ErisQuerier,
    token_supply: HashMap<Addr, Uint256>,
    generator_lp_stake: HashMap<(Addr, Addr), Uint256>,
    pools: HashMap<Addr, PoolStruct>
}

#[derive(Clone, Default)]
pub struct ErisQuerier {
    // this lets us iterate over all pairs that match the first string
    contract: String,
    value: Decimal256,
}

#[derive(Clone)]
pub struct PoolStruct {
    // this lets us iterate over all pairs that match the first string
    pub assets: Vec<(String, Uint256)>,
    pub lp: Addr
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<TerraQueryWrapper> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<TerraQueryWrapper>) -> Self {
        WasmMockQuerier {
            base,
            eris_querier: ErisQuerier::default(),
            token_supply: HashMap::new(),
            generator_lp_stake: HashMap::new(),
            pools: HashMap::new()

        }
    }

    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Custom(TerraQueryWrapper {
                query_data: _,
                route: _,
            }) => {
                panic!("DO NOT ENTER HERE")
            }
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                match from_binary(msg).unwrap() {
                    AvaiableQueries::State {} => {
                        if self.eris_querier.contract == contract_addr.to_string() {
                            SystemResult::Ok(ContractResult::from(to_binary(
                                &QueryErisHubResponse::State {
                                    exchange_rate: self.eris_querier.value,
                                },
                            )))
                        } else {
                            SystemResult::Err(SystemError::InvalidRequest {
                                error: "No borrow rate exists".to_string(),
                                request: msg.as_slice().into(),
                            })
                        }
                    
                    },
                    AvaiableQueries::Pair{} => {

                        let pool = self.pools.get(&Addr::unchecked(contract_addr)).unwrap();
                        let mut asset_infos: Vec<AssetInfo> = vec![];
                        for (contract, _) in pool.clone().assets {
                            asset_infos.push(AssetInfo::NativeToken { denom: contract })
                        }
                    
                        SystemResult::Ok(ContractResult::from(to_binary(
                            &PairInfo {
                                asset_infos: asset_infos,
                                contract_addr: Addr::unchecked(contract_addr),
                                liquidity_token: Addr::unchecked(pool.lp.clone()),
                                pair_type: PairType::Xyk {  },
                            },
                        )))
                    },
                    AvaiableQueries::Pool {  } => {

                        let pool = self.pools.get(&Addr::unchecked(contract_addr)).unwrap();
                        let mut assets: Vec<Asset> = vec![];

                        for (contract, amount) in pool.clone().assets {
                            assets.push(Asset { info: AssetInfo::NativeToken { denom: contract }, amount: Uint128::from(amount) })
                        }

                        SystemResult::Ok(ContractResult::from(to_binary(
                            &PoolResponse {
                                assets,
                                total_share: Uint128::from(self.token_supply.get(&pool.lp).unwrap().to_owned()),
                            },
                        )))

                    },
                    AvaiableQueries::Deposit { lp_token, user } => {

                        let staked = Uint128::from(self.generator_lp_stake.get(&(Addr::unchecked(user),Addr::unchecked(lp_token))).unwrap().to_owned());

                        SystemResult::Ok(ContractResult::from(to_binary(
                            &staked
                        )))

                    },
                    
                    AvaiableQueries::TokenInfo {} => {

                        SystemResult::Ok(ContractResult::from(to_binary(
                            &TokenInfoResponse {
                                name: "not_defined".to_string(),
                                symbol: "not_defined".to_string(),
                                decimals: 6_u8,
                                total_supply: Uint128::from(self.token_supply.get(&Addr::unchecked(contract_addr)).unwrap().to_owned()),
                            },
                        )))
                    }
                }
            }

            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {

    pub fn set_eris_querier(&mut self, contract: String, value: Decimal256) {
        self.eris_querier = ErisQuerier { contract, value }
    }

    pub fn set_token_supply(&mut self, contract:Addr, amount:Uint256) {
        self.token_supply.insert(contract, amount);
    }

    pub fn set_generator_lp_stake(&mut self, user:Addr, lp_contract: Addr, amount:Uint256) {
        self.generator_lp_stake.insert((user, lp_contract), amount);
    }

    pub fn set_pool_info(&mut self, pool:Addr, info:PoolStruct) {
        self.pools.insert(pool, info);

    }
}
