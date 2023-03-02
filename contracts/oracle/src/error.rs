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

    #[error("Asset is not whitelisted")]
    AssetIsNotWhitelisted {},

    #[error("Zero Price is not allowed")]
    NotValidZeroPrice {},
}
