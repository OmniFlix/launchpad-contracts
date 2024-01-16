use std::convert::Infallible;

use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;
use whitelist_types::Round;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid member limit")]
    InvalidMemberLimit {},

    #[error("Invalid start time")]
    InvalidStartTime {},

    #[error("Invalid per address limit")]
    InvalidPerAddressLimit {},

    #[error("Invalid mint price")]
    InvalidMintPrice {},

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("Invalid end time")]
    InvalidEndTime {},

    #[error("Whitelist frozen")]
    WhitelistFrozen {},

    #[error("Round limit is reached")]
    RoundReachedMintLimit {},

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

    #[error("Invalid round time")]
    InvalidRoundTime { round: Round },

    #[error("Rounds overlapped")]
    RoundsOverlapped {},

    #[error("Empty address list")]
    EmptyAddressList {},

    #[error("No active round")]
    NoActiveRound {},

    #[error("Overflow error")]
    OverflowError {},
}
impl From<ContractError> for StdError {
    fn from(err: ContractError) -> StdError {
        StdError::generic_err(err.to_string())
    }
}
impl From<Infallible> for ContractError {
    fn from(_err: Infallible) -> Self {
        ContractError::OverflowError {}
    }
}
