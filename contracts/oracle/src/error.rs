use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Asset already whitelisted")]
    AssetAlreadyWhitelisted {},

    #[error("Asset {asset:?} is not whitelisted")]
    AssetIsNotWhitelisted { asset: String },

    #[error("Zero Price is not allowed")]
    NotValidZeroPrice {},

    #[error("Price source is not feeder for asset {asset:?}")]
    SourceIsNotFeeder { asset: String },

    #[error("Price has never been feeded")]
    PriceNeverFeeded {},

    #[error("Wrong price source")]
    WrongPriceSource {},

    #[error("Pool must have two assets")]
    PoolInvalidAssetsLenght {},
}
