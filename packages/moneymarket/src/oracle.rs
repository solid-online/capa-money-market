use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::math::Decimal256;

use cosmwasm_std::{Addr, Binary};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String,
    pub base_asset: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<String>,
    },

    FeedPrice {
        prices: Vec<(String, Decimal256)>, // (asset, price)
    },

    UpdateSource {
        asset: String,
        source: UpdateSource,
    },

    RegisterAsset {
        asset: String,
        source: RegisterSource,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    SourceInfo {
        asset: String,
    },
    Price {
        base: String,
        quote: String,
    },
    Prices {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[allow(unreachable_patterns)]
pub enum Source {
    Feeder {
        feeder: Addr,
        price: Option<Decimal256>,
        last_updated_time: Option<u64>,
    },
    LsdContractQuery {
        base_asset: String,
        contract: Addr,
        query_msg: Binary,
        path_key: Vec<String>,
        is_inverted: bool,
    },
    AstroportLpVault {
        vault_contract: Addr,
        generator_contract: Addr,
        pool_contract: Addr,
        lp_contract: Addr,
        assets: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[allow(unreachable_patterns)]
pub enum RegisterSource {
    Feeder {
        feeder: Addr,
    },
    LsdContractQuery {
        base_asset: String,
        contract: Addr,
        query_msg: Binary,
        path_key: Vec<String>,
        is_inverted: bool,
    },
    AstroportLpVault {
        vault_contract: Addr,
        generator_contract: Addr,
        pool_contract: Addr,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[allow(unreachable_patterns)]
pub enum UpdateSource {
    Feeder {
        feeder: Addr,
    },
    LsdContractQuery {
        base_asset: Option<String>,
        contract: Option<Addr>,
        query_msg: Option<Binary>,
        path_key: Option<Vec<String>>,
        is_inverted: Option<bool>,
    },
    AstroportLpVault {
        vault_contract: Option<Addr>,
        generator_contract: Option<Addr>,
        pool_contract: Option<Addr>,
    },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub base_asset: String,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SourceInfoResponse {
    pub source: Source,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PriceResponse {
    pub rate: Decimal256,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PricesResponseElem {
    pub asset: String,
    pub price: Decimal256,
    pub last_updated_time: u64,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PricesResponse {
    pub prices: Vec<PricesResponseElem>,
}
