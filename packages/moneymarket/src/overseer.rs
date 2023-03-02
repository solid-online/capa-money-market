use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::tokens::TokensHuman;
use cosmwasm_bignumber::math::{Decimal256, Uint256};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    /// Initial owner address
    pub owner_addr: String,
    /// Oracle contract address for collateral tokens
    pub oracle_contract: String,
    /// Market contract address to receive missing interest buffer
    pub market_contract: String,
    /// Liquidation model contract address to compute liquidation amount
    pub liquidation_contract: String,
    /// Collector contract address which is collect fees from protocol and divide them between the team and the stakers
    pub collector_contract: String,
    /// The base denomination used when fetching oracle price,
    /// reward distribution, and borrow
    pub stable_contract: String,

    pub price_timeframe: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    ////////////////////
    /// Owner operations
    ////////////////////
    /// Update Configs
    UpdateConfig {
        owner_addr: Option<String>,
        oracle_contract: Option<String>,
        liquidation_contract: Option<String>,
        price_timeframe: Option<u64>,
    },
    /// Create new custody contract for the given collateral token
    Whitelist {
        name: String,             // bAsset name
        symbol: String,           // bAsset symbol
        collateral_token: String, // bAsset token contract
        custody_contract: String, // bAsset custody contract
        max_ltv: Decimal256,      // Loan To Value ratio
    },
    /// Update registered whitelist info
    UpdateWhitelist {
        collateral_token: String,         // bAsset token contract
        custody_contract: Option<String>, // bAsset custody contract
        max_ltv: Option<Decimal256>,      // Loan To Value ratio
    },

    ////////////////////
    /// User operations
    ////////////////////
    LockCollateral {
        collaterals: TokensHuman, // <(Collateral Token, Amount)>
    },
    UnlockCollateral {
        collaterals: TokensHuman, // <(Collateral Token, Amount)>
    },

    /////////////////////////////
    /// Permissionless operations
    /////////////////////////////
    LiquidateCollateral { borrower: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    FundReserve {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Whitelist {
        collateral_token: Option<String>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    Collaterals {
        borrower: String,
    },
    AllCollaterals {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    BorrowLimit {
        borrower: String,
        block_time: Option<u64>,
    },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ConfigResponse {
    pub owner_addr: String,
    pub oracle_contract: String,
    pub market_contract: String,
    pub liquidation_contract: String,
    pub collector_contract: String,
    pub stable_contract: String,
    pub price_timeframe: u64,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct WhitelistResponseElem {
    pub name: String,
    pub symbol: String,
    pub max_ltv: Decimal256,
    pub custody_contract: String,
    pub collateral_token: String,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct WhitelistResponse {
    pub elems: Vec<WhitelistResponseElem>,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CollateralsResponse {
    pub borrower: String,
    pub collaterals: TokensHuman, // <(Collateral Token, Amount)>
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AllCollateralsResponse {
    pub all_collaterals: Vec<CollateralsResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct BorrowLimitResponse {
    pub borrower: String,
    pub borrow_limit: Uint256,
}
