use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Api, BalanceResponse, BankQuery, CanonicalAddr, Coin,
    ContractResult, OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError, SystemResult,
    Uint128, WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;
use cw20::{Cw20QueryMsg, TokenInfoResponse};
use std::collections::HashMap;
use std::marker::PhantomData;
use terra_cosmwasm::TerraQueryWrapper;

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
    cw20_balance: Uint128,
    uluna_balance: Uint128,
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    // this lets us iterate over all pairs that match the first string
    balances: HashMap<String, HashMap<String, Uint128>>,
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
            QueryRequest::Wasm(WasmQuery::Raw { contract_addr, key }) => {
                let key: &[u8] = key.as_slice();

                let prefix_token_info = to_length_prefixed(b"token_info").to_vec();
                let prefix_balance = to_length_prefixed(b"balance").to_vec();

                let balances: &HashMap<String, Uint128> =
                    match self.token_querier.balances.get(contract_addr) {
                        Some(balances) => balances,
                        None => {
                            return SystemResult::Err(SystemError::InvalidRequest {
                                error: format!(
                                    "No balance info exists for the contract {}",
                                    contract_addr
                                ),
                                request: key.into(),
                            })
                        }
                    };

                if key.to_vec() == prefix_token_info {
                    let mut total_supply = Uint128::zero();

                    for balance in balances {
                        total_supply += *balance.1;
                    }

                    SystemResult::Ok(ContractResult::from(to_binary(
                        &to_binary(&TokenInfoResponse {
                            name: "mAPPL".to_string(),
                            symbol: "mAPPL".to_string(),
                            decimals: 6,
                            total_supply,
                        })
                        .unwrap(),
                    )))
                } else if key[..prefix_balance.len()].to_vec() == prefix_balance {
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
                    let balance = match balances.get(&address.to_string()) {
                        Some(v) => v,
                        None => {
                            return SystemResult::Err(SystemError::InvalidRequest {
                                error: "Balance not found".to_string(),
                                request: key.into(),
                            })
                        }
                    };
                    SystemResult::Ok(ContractResult::from(to_binary(
                        &to_binary(&balance).unwrap(),
                    )))
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }

            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: _,
                msg,
            }) => match from_binary(msg).unwrap() {
                Cw20QueryMsg::Balance { address: _ } => {
                    SystemResult::Ok(ContractResult::from(to_binary(&cw20::BalanceResponse {
                        balance: self.cw20_balance,
                    })))
                }
                Cw20QueryMsg::TokenInfo {} => {
                    SystemResult::Ok(ContractResult::from(to_binary("not implemented")))
                }
                Cw20QueryMsg::Allowance {
                    owner: _,
                    spender: _,
                } => SystemResult::Ok(ContractResult::from(to_binary("not implemented"))),
                Cw20QueryMsg::Minter {} => {
                    SystemResult::Ok(ContractResult::from(to_binary("not implemented")))
                }
                Cw20QueryMsg::MarketingInfo {} => {
                    SystemResult::Ok(ContractResult::from(to_binary("not implemented")))
                }
                Cw20QueryMsg::DownloadLogo {} => {
                    SystemResult::Ok(ContractResult::from(to_binary("not implemented")))
                }
                Cw20QueryMsg::AllAllowances {
                    owner: _,
                    start_after: _,
                    limit: _,
                } => SystemResult::Ok(ContractResult::from(to_binary("not implemented"))),
                Cw20QueryMsg::AllAccounts {
                    start_after: _,
                    limit: _,
                } => SystemResult::Ok(ContractResult::from(to_binary("not implemented"))),
            },

            QueryRequest::Bank(BankQuery::Balance { address: _, denom }) => {
                let bank_res = BalanceResponse {
                    amount: Coin {
                        denom: denom.to_string(),
                        amount: self.uluna_balance,
                    },
                };
                SystemResult::Ok(ContractResult::from(to_binary(&bank_res)))
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
            cw20_balance: Uint128::zero(),
            uluna_balance: Uint128::zero(),
        }
    }
}
