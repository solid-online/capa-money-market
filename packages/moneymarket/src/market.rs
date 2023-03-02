use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::math::{Decimal256, Uint256};
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    /// Owner address for config update
    pub owner_addr: String,
    /// Cw20 code id for Solid
    pub stable_code_id: u64,
    /// Base borrow fee:
    pub base_borrow_fee: Decimal256,
    // Base fee increase factor
    pub fee_increase_factor: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),

    ////////////////////
    /// Owner operations
    ////////////////////
    /// Register Contracts contract address
    RegisterContracts {
        overseer_contract: String,
        /// The contract has the logics for
        /// Collector contract to send all the reserve
        collector_contract: String,
        /// Faucet contract to drip CAPA token to users
        liquidation_contract: String,

        oracle_contract: String,
    },

    /// Update config values
    UpdateConfig {
        owner_addr: Option<String>,
        liquidation_contract: Option<String>,
        base_borrow_fee: Option<Decimal256>,
        fee_increase_factor: Option<Decimal256>,
    },

    /// Borrow stable asset with collaterals in overseer contract
    BorrowStable {
        borrow_amount: Uint256,
        to: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// Return stable coins to a user
    /// according to exchange rate
    RepayStable {},
    RepayStableFromLiquidation {
        borrower: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    BorrowerInfo {
        borrower: String,
    },
    BorrowerInfos {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ConfigResponse {
    pub owner_addr: String,
    pub stable_contract: String,
    pub overseer_contract: String,
    pub collector_contract: String,
    pub liquidation_contract: String,
    pub oracle_contract: String,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct StateResponse {
    pub total_liabilities: Decimal256,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct BorrowerInfoResponse {
    pub borrower: String,
    pub loan_amount: Uint256,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct BorrowerInfosResponse {
    pub borrower_infos: Vec<BorrowerInfoResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}
