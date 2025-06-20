use cosmwasm_std::{OverflowError, StdError, Uint256};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Borrow amount too high; Loan liability becomes greater than borrow limit: {0}")]
    BorrowExceedsLimit(Uint256),

    #[error("Must deposit initial funds {0}{0}")]
    InitialFundsNotDeposited(Uint256, String),

    #[error("Invalid reply ID")]
    InvalidReplyId {},

    #[error("Exceeds {0} max borrow factor; borrow demand too high")]
    MaxBorrowFactorReached(String),

    #[error("Invalid request: \"redeem stable\" message not included in request")]
    MissingRedeemStableHook {},

    #[error("Not enough {0} available; borrow demand too high")]
    NoStableAvailable(String),

    #[error("Deposit amount must be greater than 0 {0}")]
    ZeroDeposit(String),

    #[error("Repay amount must be greater than 0 {0}")]
    ZeroRepay(String),
}
