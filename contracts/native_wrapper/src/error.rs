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

    #[error("Invalid Zero Amount")]
    InvalidZeroAmount {},

    #[error("Invalid reply ID")]
    InvalidReplyId {},

    #[error("Invalid request: \"redeem stable\" message not included in request")]
    MissingRedeemStableHook {},

    #[error("Deposit amount must be greater than 0 {0}")]
    ZeroDeposit(String),

    #[error("Repay amount must be greater than 0 {0}")]
    ZeroRepay(String),
    #[error("Must send only one coin")]
    TooManyCoins(),
}
