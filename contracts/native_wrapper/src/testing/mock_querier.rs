use cosmwasm_bignumber::math::Decimal256;
use moneymarket::oracle::PriceResponse;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Coin, ContractResult, OwnedDeps, Querier, QuerierResult,
    QueryRequest, SystemError, SystemResult, WasmQuery,
};

use core::panic;
use std::collections::HashMap;
use std::marker::PhantomData;

use terra_cosmwasm::TerraQueryWrapper;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Query oracle price to oracle contract
    Price { base: String, quote: String },
}

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies(
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
    oracle_price_querier: OraclePriceQuerier,
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

#[derive(Clone, Default)]
pub struct OraclePriceQuerier {
    // this lets us iterate over all pairs that match the first string
    oracle_price: HashMap<(String, String), (Decimal256, u64, u64)>,
}

#[allow(clippy::type_complexity)]
impl OraclePriceQuerier {
    pub fn new(oracle_price: &[(&(String, String), &(Decimal256, u64, u64))]) -> Self {
        OraclePriceQuerier {
            oracle_price: oracle_price_to_map(oracle_price),
        }
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn oracle_price_to_map(
    oracle_price: &[(&(String, String), &(Decimal256, u64, u64))],
) -> HashMap<(String, String), (Decimal256, u64, u64)> {
    let mut oracle_price_map: HashMap<(String, String), (Decimal256, u64, u64)> = HashMap::new();
    for (base_quote, oracle_price) in oracle_price.iter() {
        oracle_price_map.insert((*base_quote).clone(), **oracle_price);
    }

    oracle_price_map
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Custom(TerraQueryWrapper {
                query_data: _,
                route: _,
            }) => {
                panic!("DO NOT ENTER HERE")
            }
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: _,
                msg,
            }) => match from_binary(msg).unwrap() {
                QueryMsg::Price { base, quote } => {
                    match self.oracle_price_querier.oracle_price.get(&(base, quote)) {
                        Some(v) => {
                            SystemResult::Ok(ContractResult::from(to_binary(&PriceResponse {
                                rate: v.0,
                                last_updated_base: v.1,
                                last_updated_quote: v.2,
                            })))
                        }
                        None => SystemResult::Err(SystemError::InvalidRequest {
                            error: "No oracle price exists".to_string(),
                            request: msg.as_slice().into(),
                        }),
                    }
                }
            },
            QueryRequest::Wasm(WasmQuery::Raw {
                contract_addr: _,
                key: _,
            }) => {
                panic!("DO NOT ENTER HERE")
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<TerraQueryWrapper>) -> Self {
        WasmMockQuerier {
            base,
            oracle_price_querier: OraclePriceQuerier::default(),
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn with_oracle_price(
        &mut self,
        oracle_price: &[(&(String, String), &(Decimal256, u64, u64))],
    ) {
        self.oracle_price_querier = OraclePriceQuerier::new(oracle_price);
    }
}
