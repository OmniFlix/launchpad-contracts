use cosmwasm_std::{Coin, StdError, Timestamp, Uint128};
use cw_utils::PaymentError;
use minter_types::{config::ConfigurationError, token_details::TokenDetailsError};
use pauser::PauseError;
use serde_json::Error as SerdeError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error(transparent)]
    Pause(#[from] PauseError),

    #[error(transparent)]
    TokenDetailsError(#[from] TokenDetailsError),

    #[error(transparent)]
    ConfigurationError(#[from] ConfigurationError),

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

    #[error("Incorrect payment amount")]
    IncorrectPaymentAmount { expected: Uint128, sent: Uint128 },

    #[error("No tokens left to mint")]
    NoTokensLeftToMint {},

    #[error("Address has reached the mint limit")]
    AddressReachedMintLimit {},

    #[error("Token id is not mintable")]
    TokenIdNotMintable {},

    #[error("Address is not whitelisted")]
    AddressNotWhitelisted {},

    #[error("Whitelist is not active")]
    WhitelistNotActive {},

    #[error("Whitelist is already active")]
    WhitelistAlreadyActive {},

    #[error("Error saving tokens")]
    ErrorSavingTokens {},

    #[error("Public minting ended")]
    PublicMintingEnded {},

    #[error("Token limit reached")]
    TokenLimitReached {},

    #[error("Token limit not set")]
    TokenLimitNotSet {},

    #[error("No mint_instance available")]
    NoMintInstanceAvailable {},

    #[error("Invalid mint_instance id")]
    InvalidMintInstanceId {},

    #[error("MintInstance cannot be removed, tokens are minted from this mint_instance")]
    MintInstanceCantBeRemoved,
}
impl From<ContractError> for StdError {
    fn from(err: ContractError) -> StdError {
        StdError::generic_err(err.to_string())
    }
}

impl From<SerdeError> for ContractError {
    fn from(err: SerdeError) -> ContractError {
        ContractError::Std(StdError::generic_err(err.to_string()))
    }
}
