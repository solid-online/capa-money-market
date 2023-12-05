use cosmwasm_schema::cw_serde;

use cosmwasm_bignumber::math::Decimal256;

use cosmwasm_std::{Addr, Empty, QueryRequest};

/// Base precision of assets.
///
/// During the registration of a new `asset`, in case the `Soruce::Feeder`, subtract the `asset` precision with `BASE_PRECISION`
pub const BASE_PRECISION: u8 = 6;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub base_asset: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<String>,
    },

    FeedPrice {
        prices: Vec<FeedPriceInfo>, // (asset, price)
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

#[cw_serde]
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

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct FeedPriceInfo {
    pub asset_name: String,
    pub price: Decimal256,
}

impl From<(String, Decimal256)> for FeedPriceInfo {
    fn from(value: (String, Decimal256)) -> Self {
        Self {
            asset_name: value.0,
            price: value.1,
        }
    }
}

#[cw_serde]
pub enum Source {
    Feeder {
        feeder: Addr,
        price: Option<Decimal256>,
        last_updated_time: Option<u64>,
        normalized_precision: u8,
    },
    OnChainQuery {
        base_asset: Option<String>,
        query: QueryRequest<Empty>,
        path_key: Vec<PathKey>,
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

#[cw_serde]
pub enum RegisterSource {
    Feeder {
        feeder: String,

        precision: u8,
    },
    OnChainRate {
        base_asset: Option<String>,
        query: QueryRequest<Empty>,
        path_key: Vec<PathKey>,
        is_inverted: bool,
    },
    AstroportLpVault {
        vault_contract: Addr,
        generator_contract: Addr,
        pool_contract: Addr,
    },
}

#[cw_serde]
pub enum UpdateSource {
    Feeder {
        feeder: Addr,
        precision: u8,
    },
    OnChainRate {
        base_asset: Option<UpdateOption<String>>,
        query: Option<QueryRequest<Empty>>,
        path_key: Option<Vec<PathKey>>,
        is_inverted: Option<bool>,
    },
    AstroportLpVault {
        vault_contract: Option<Addr>,
        generator_contract: Option<Addr>,
        pool_contract: Option<Addr>,
    },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub base_asset: String,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct SourceInfoResponse {
    pub source: Source,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct PriceResponse {
    pub rate: Decimal256,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct PricesResponseElem {
    pub asset: String,
    pub price: Decimal256,
    pub last_updated_time: u64,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct PricesResponse {
    pub prices: Vec<PricesResponseElem>,
}

#[cw_serde]
pub enum PathKey {
    Index(u64),
    String(String),
}

#[cw_serde]
pub enum UpdateOption<T> {
    ToNone,
    Some(T),
}

impl<T: Clone> UpdateOption<T> {
    pub fn into_option(&self) -> Option<T> {
        match self {
            UpdateOption::ToNone => None,
            UpdateOption::Some(t) => Some(t.clone()),
        }
    }

    pub fn unwrap(self) -> T {
        match self {
            UpdateOption::ToNone => panic!("Unwrap a None value"),
            UpdateOption::Some(val) => val,
        }
    }
}

#[allow(clippy::from_over_into)]
impl<T> Into<Option<T>> for UpdateOption<T> {
    fn into(self) -> Option<T> {
        match self {
            UpdateOption::ToNone => None,
            UpdateOption::Some(val) => Some(val),
        }
    }
}
