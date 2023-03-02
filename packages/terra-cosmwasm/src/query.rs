use crate::route::TerraRoute;
use cosmwasm_std::{Coin, CustomQuery, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// TerraQueryWrapper is an override of QueryRequest::Custom to access Terra-specific modules
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TerraQueryWrapper {
    pub route: TerraRoute,
    pub query_data: TerraQuery,
}
// implement custom query
impl CustomQuery for TerraQueryWrapper {}
/// TerraQuery is defines available query datas
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TerraQuery {
    Swap {
        offer_coin: Coin,
        ask_denom: String,
    },

    ExchangeRates {
        base_denom: String,
        quote_denoms: Vec<String>,
    },
    ContractInfo {
        contract_address: String,
    },
}
/// SwapResponse is data format returned from SwapRequest::Simulate query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SwapResponse {
    pub receive: Coin,
}
/// ExchangeRateItem is data format returned from OracleRequest::ExchangeRates query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ExchangeRateItem {
    pub quote_denom: String,
    pub exchange_rate: Decimal,
}
/// ExchangeRatesResponse is data format returned from OracleRequest::ExchangeRates query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ExchangeRatesResponse {
    pub base_denom: String,
    pub exchange_rates: Vec<ExchangeRateItem>,
}
/// ContractInfoResponse is data format returned from WasmRequest::ContractInfo query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ContractInfoResponse {
    pub address: String,
    pub creator: String,
    pub code_id: u64,
    pub admin: Option<String>,
}
