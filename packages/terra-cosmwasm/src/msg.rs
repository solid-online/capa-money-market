use astroport::asset::{Asset, AssetInfo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::math::{Decimal256, Uint256};
use cosmwasm_std::{Coin, CosmosMsg};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// TerraMsgWrapper is an override of CosmosMsg::Custom to show this works and can be extended in the contract
pub struct TerraMsgWrapper {
    pub msg_data: TerraMsg,
}
impl cosmwasm_std::CustomMsg for TerraMsgWrapper {}

// this is a helper to be able to return these as CosmosMsg easier
impl From<TerraMsgWrapper> for CosmosMsg<TerraMsgWrapper> {
    fn from(original: TerraMsgWrapper) -> Self {
        CosmosMsg::Custom(original)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TerraMsg {
    Swap {
        belief_price: String,
        max_spread: String,
        offer_asset: Asset,
    },
    SwapSend {
        to_address: String,
        offer_coin: Coin,
        ask_denom: String,
    },
}

// create_swap_msg returns wrapped swap msg
pub fn create_swap_msg(amount: Uint256, belief_price: Decimal256) -> CosmosMsg<TerraMsgWrapper> {
    cosmwasm_std::CosmosMsg::Custom(TerraMsgWrapper {
        msg_data: TerraMsg::Swap {
            belief_price: belief_price.to_string(),
            max_spread: "3".to_string(),
            offer_asset: Asset {
                amount: amount.into(),
                info: AssetInfo::NativeToken {
                    denom: "uluna".to_string(),
                },
            },
        },
    })
}
