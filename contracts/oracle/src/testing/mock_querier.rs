use std::marker::PhantomData;

use cosmwasm_bignumber::math::Decimal256;
use cosmwasm_std::{Coin, testing::{MockStorage, MockApi, MOCK_CONTRACT_ADDR, MockQuerier}, OwnedDeps, Querier, QuerierResult, QueryRequest, from_slice, SystemResult, SystemError, WasmQuery, from_binary, to_binary, ContractResult};
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use terra_cosmwasm::TerraQueryWrapper;



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryErisHub {
    State {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryErisHubResponse {
    State {exchange_rate:Decimal256},
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
    eris_querier: ErisQuerier
}


#[derive(Clone, Default)]
pub struct ErisQuerier {
    // this lets us iterate over all pairs that match the first string
    contract: String,
    value: Decimal256
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
                    QueryErisHub::State {} => {
                       if self.eris_querier.contract == contract_addr.as_ref() {SystemResult::Ok(ContractResult::from(to_binary(
                                &QueryErisHubResponse::State { exchange_rate: self.eris_querier.value },
                            ))) } else {
                             SystemResult::Err(SystemError::InvalidRequest {
                                error: "No borrow rate exists".to_string(),
                                request: msg.as_slice().into(),
                            })}
                        }
                    }
                 
                    
                
            },
       
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<TerraQueryWrapper>) -> Self {
        WasmMockQuerier {
            base,
            eris_querier: ErisQuerier::default()
        }
    }

    pub fn set_eris_querier(&mut self, contract:String, value:Decimal256) {
        self.eris_querier = ErisQuerier{contract: contract, value:value}
    }
}
