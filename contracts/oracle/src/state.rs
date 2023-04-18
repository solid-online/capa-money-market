use cosmwasm_bignumber::math::Decimal256;
use cw_storage_plus::{Item, Map};
use moneymarket::oracle::Source;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;

pub const CONFIG: Item<Config> = Item::new("config");
pub const ASSETS: Map<String, Source> = Map::new("assets");

/// --- STRUCTURES ---

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub base_asset: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PriceData {
    pub source: Source,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PriceInfo {
    pub price: Decimal256,
    pub last_updated_time: u64,
}
