use cosmwasm_std::{Coin, StdError, Timestamp, Uint128};
use cw_utils::PaymentError;
use pauser::PauseError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error(transparent)]
    Pause(#[from] PauseError),

    #[error("Payment error")]
    PaymentError(#[from] PaymentError),

    #[error("Overflow error")]
    OverflowError {},

    #[error("Divide by zero")]
    DivideByZero {},

    #[error("Invalid creation fee")]
    InvalidCreationFee {
        expected: Vec<Coin>,
        sent: Vec<Coin>,
    },

    #[error("Minting has not started yet")]
    MintingNotStarted {
        start_time: Timestamp,
        current_time: Timestamp,
    },

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

    #[error("Invalid end time")]
    InvalidEndTime {},

    #[error("Address is not whitelisted")]
    AddressNotWhitelisted {},

    #[error("Whitelist is not active")]
    WhitelistNotActive {},

    #[error("Whitelist is already active")]
    WhitelistAlreadyActive {},

    #[error("Round start time is invalid")]
    RoundStartTimeInvalid {},

    #[error("Error saving tokens")]
    ErrorSavingTokens {},

    #[error("Public minting ended")]
    PublicMintingEnded {},

    #[error("Token limit reached")]
    TokenLimitReached {},

    #[error("Token limit not set")]
    TokenLimitNotSet {},
}
impl From<ContractError> for StdError {
    fn from(err: ContractError) -> StdError {
        StdError::generic_err(err.to_string())
    }
}
