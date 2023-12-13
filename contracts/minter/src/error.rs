use std::convert::Infallible;

use cosmwasm_std::{CheckedFromRatioError, ConversionOverflowError, StdError, Uint128};
use cw_utils::PaymentError;
use thiserror::Error;

use crate::state::Round;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Payment error")]
    PaymentError(#[from] PaymentError),

    #[error("Overflow error")]
    OverflowError {},

    #[error("Divide by zero")]
    DivideByZero {},

    #[error("Invalid creation fee")]
    InvalidCreationFee { expected: Uint128, sent: Uint128 },

    #[error("Minting has not started yet")]
    MintingNotStarted { start_time: u64, current_time: u64 },

    #[error("Minting has already started")]
    MintingAlreadyStarted {},

    #[error("Incorrect payment amount")]
    IncorrectPaymentAmount { expected: Uint128, sent: Uint128 },

    #[error("No tokens left to mint")]
    NoTokensLeftToMint {},

    #[error("Address has reached the mint limit")]
    AddressReachedMintLimit {},

    #[error("Token id is not mintable")]
    TokenIdNotMintable {},

    #[error("Per address limit cannot be zero")]
    PerAddressLimitZero {},

    #[error("Invalid number of tokens")]
    InvalidNumTokens {},

    #[error("Invalid royalty ratio")]
    InvalidRoyaltyRatio {},

    #[error("Invalid mint price")]
    InvalidMintPrice {},

    #[error("Invalid start time")]
    InvalidStartTime {},

    #[error("Whitelist address limit is reached")]
    WhitelistAddressLimitReached {},

    #[error("Address is not whitelisted")]
    AddressNotWhitelisted {},

    #[error("Whitelist is not active")]
    WhitelistNotActive {},

    #[error("Whitelist is already active")]
    WhitelistAlreadyActive {},

    #[error("Not a whitelist contract")]
    NotWhitelistContract {},

    #[error("Round limit is reached")]
    RoundReachedMintLimit {},

    #[error("Rounds overlaped")]
    RoundsOverlaped { round: Round },

    #[error("Round is not active")]
    RoundEnded {},

    #[error("Round has already started")]
    RoundAlreadyStarted {},

    #[error("Round start time is invalid")]
    RoundStartTimeInvalid {},

    #[error("Collection not found")]
    CollectionNotFound {},

    #[error("Round does not exist")]
    RoundNotFound {},

    #[error("Round already exists")]
    RoundAlreadyExists {},

    #[error("Invalid roud type")]
    InvalidRoundType { expected: String, actual: String },

    #[error("Round start time can not be later than end time")]
    InvalidRoundTime { round: Round },
}

impl From<ContractError> for StdError {
    fn from(err: ContractError) -> StdError {
        StdError::generic_err(err.to_string())
    }
}
impl From<CheckedFromRatioError> for ContractError {
    fn from(_err: CheckedFromRatioError) -> Self {
        ContractError::DivideByZero {}
    }
}

impl From<Infallible> for ContractError {
    fn from(_err: Infallible) -> Self {
        ContractError::OverflowError {}
    }
}
impl From<ConversionOverflowError> for ContractError {
    fn from(_err: ConversionOverflowError) -> Self {
        ContractError::OverflowError {}
    }
}
