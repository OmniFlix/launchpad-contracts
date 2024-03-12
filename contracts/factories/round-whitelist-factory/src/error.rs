use cosmwasm_std::{Coin, StdError};
use cw_utils::PaymentError;
use pauser::PauseError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]

pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Payment error")]
    PaymentError(#[from] PaymentError),

    #[error(transparent)]
    Pause(#[from] PauseError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid minter code id")]
    InvalidMinterCodeId {},

    #[error("Inncorrect funds")]
    IncorrectFunds {
        expected: Vec<Coin>,
        actual: Vec<Coin>,
    },
    #[error("Invalid Mint Denom")]
    InvalidMintDenom {},

    #[error("Mint denom not allowed")]
    MintDenomNotAllowed {},

    #[error("Missing creation fee")]
    MissingCreationFee {},

    #[error("Missing minter creation fee")]
    MissingMinterCreationFee {},
}

impl From<ContractError> for StdError {
    fn from(err: ContractError) -> StdError {
        StdError::generic_err(err.to_string())
    }
}
