use moneymarket::oracle::PriceResponse;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::math::{Decimal256, Uint256};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Api, CanonicalAddr, Coin, ContractResult, OwnedDeps,
    Querier, QuerierResult, QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;
use std::collections::HashMap;
use std::marker::PhantomData;

use cw20::TokenInfoResponse;
use moneymarket::interest_model::BorrowRateResponse;
use moneymarket::overseer::{BorrowLimitResponse, ConfigResponse};
use terra_cosmwasm::TerraQueryWrapper;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Query borrow rate to interest model contract
    BorrowRate {},
    /// Query borrow limit to overseer contract
    BorrowLimit {
        borrower: String,
        block_time: Option<u64>,
    },
    /// Query overseer config to get target deposit rate
    Config {},

    Price {
        base: String,
        quote: String,
    },
    /// Query cw20 Token Info
    TokenInfo {},
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
    token_querier: TokenQuerier,
    borrow_rate_querier: BorrowRateQuerier,
    borrow_limit_querier: BorrowLimitQuerier,
    oracle_price_querier: OraclePriceQuerier,
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    // this lets us iterate over all pairs that match the first string
    balances: HashMap<String, HashMap<String, Uint128>>,
}

impl TokenQuerier {}

#[derive(Clone, Default)]
pub struct TaxQuerier {}

#[derive(Clone, Default)]
pub struct BorrowRateQuerier {
    // this lets us iterate over all pairs that match the first string
    borrower_rate: HashMap<String, Decimal256>,
}

impl BorrowRateQuerier {
    pub fn new(borrower_rate: &[(&String, &Decimal256)]) -> Self {
        BorrowRateQuerier {
            borrower_rate: borrower_rate_to_map(borrower_rate),
        }
    }
}

pub(crate) fn borrower_rate_to_map(
    borrower_rate: &[(&String, &Decimal256)],
) -> HashMap<String, Decimal256> {
    let mut borrower_rate_map: HashMap<String, Decimal256> = HashMap::new();
    for (market_contract, borrower_rate) in borrower_rate.iter() {
        borrower_rate_map.insert((*market_contract).clone(), **borrower_rate);
    }
    borrower_rate_map
}

#[derive(Clone, Default)]
pub struct BorrowLimitQuerier {
    // this lets us iterate over all pairs that match the first string
    borrow_limit: HashMap<String, Uint256>,
}

impl BorrowLimitQuerier {
    pub fn new(borrow_limit: &[(&String, &Uint256)]) -> Self {
        BorrowLimitQuerier {
            borrow_limit: borrow_limit_to_map(borrow_limit),
        }
    }
}

pub(crate) fn borrow_limit_to_map(
    borrow_limit: &[(&String, &Uint256)],
) -> HashMap<String, Uint256> {
    let mut borrow_limit_map: HashMap<String, Uint256> = HashMap::new();
    for (market_contract, borrow_limit) in borrow_limit.iter() {
        borrow_limit_map.insert((*market_contract).clone(), **borrow_limit);
    }
    borrow_limit_map
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
                    QueryMsg::BorrowRate {} => {
                        match self.borrow_rate_querier.borrower_rate.get(contract_addr) {
                            Some(v) => SystemResult::Ok(ContractResult::from(to_binary(
                                &BorrowRateResponse { rate: *v },
                            ))),
                            None => SystemResult::Err(SystemError::InvalidRequest {
                                error: "No borrow rate exists".to_string(),
                                request: msg.as_slice().into(),
                            }),
                        }
                    }
                    QueryMsg::BorrowLimit {
                        borrower,
                        block_time: _,
                    } => match self.borrow_limit_querier.borrow_limit.get(&borrower) {
                        Some(v) => SystemResult::Ok(ContractResult::from(to_binary(
                            &BorrowLimitResponse {
                                borrower,
                                borrow_limit: *v,
                            },
                        ))),
                        None => SystemResult::Err(SystemError::InvalidRequest {
                            error: "No borrow limit exists".to_string(),
                            request: msg.as_slice().into(),
                        }),
                    },
                    QueryMsg::Config {} => {
                        SystemResult::Ok(ContractResult::from(to_binary(&ConfigResponse {
                            owner_addr: "".to_string(),
                            oracle_contract: "".to_string(),
                            market_contract: "".to_string(),
                            liquidation_contract: "".to_string(),
                            collector_contract: "".to_string(),
                            stable_contract: "".to_string(),
                            price_timeframe: 100u64,
                        })))
                    }

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
                    QueryMsg::TokenInfo {} => {
                        let balances: HashMap<String, Uint128> =
                            match self.token_querier.balances.get(contract_addr) {
                                Some(balances) => balances.clone(),
                                None => HashMap::new(),
                            };

                        let mut total_supply = Uint128::zero();

                        for balance in balances {
                            total_supply += balance.1;
                        }

                        SystemResult::Ok(ContractResult::from(to_binary(&TokenInfoResponse {
                            name: "mAPPL".to_string(),
                            symbol: "mAPPL".to_string(),
                            decimals: 6,
                            total_supply,
                        })))
                    }
                }
            }
            QueryRequest::Wasm(WasmQuery::Raw { contract_addr, key }) => {
                let key: &[u8] = key.as_slice();

                let prefix_balance = to_length_prefixed(b"balance").to_vec();

                let balances: HashMap<String, Uint128> =
                    match self.token_querier.balances.get(contract_addr) {
                        Some(balances) => balances.clone(),
                        None => HashMap::new(),
                    };

                if key[..prefix_balance.len()].to_vec() == prefix_balance {
                    let key_address: &[u8] = &key[prefix_balance.len()..];
                    let address_raw: CanonicalAddr = CanonicalAddr::from(key_address);
                    let api: MockApi = MockApi::default();
                    let address: Addr = match api.addr_humanize(&address_raw) {
                        Ok(v) => v,
                        Err(e) => {
                            return SystemResult::Err(SystemError::InvalidRequest {
                                error: format!("Parsing query request: {}", e),
                                request: key.into(),
                            })
                        }
                    };
                    let balance = match balances.get(address.as_str()) {
                        Some(v) => v,
                        None => {
                            return SystemResult::Err(SystemError::InvalidRequest {
                                error: "Balance not found".to_string(),
                                request: key.into(),
                            })
                        }
                    };
                    SystemResult::Ok(ContractResult::from(to_binary(&balance)))
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<TerraQueryWrapper>) -> Self {
        WasmMockQuerier {
            base,
            token_querier: TokenQuerier::default(),
            borrow_rate_querier: BorrowRateQuerier::default(),
            borrow_limit_querier: BorrowLimitQuerier::default(),
            oracle_price_querier: OraclePriceQuerier::default(),
        }
    }

    pub fn with_borrow_rate(&mut self, borrow_rate: &[(&String, &Decimal256)]) {
        self.borrow_rate_querier = BorrowRateQuerier::new(borrow_rate);
    }

    pub fn with_borrow_limit(&mut self, borrow_limit: &[(&String, &Uint256)]) {
        self.borrow_limit_querier = BorrowLimitQuerier::new(borrow_limit);
    }
    #[allow(clippy::type_complexity)]
    pub fn with_oracle_price(
        &mut self,
        oracle_price: &[(&(String, String), &(Decimal256, u64, u64))],
    ) {
        self.oracle_price_querier = OraclePriceQuerier::new(oracle_price);
    }
}
