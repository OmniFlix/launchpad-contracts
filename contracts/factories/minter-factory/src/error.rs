use cosmwasm_std::{CheckedFromRatioError, ConversionOverflowError, StdError, Timestamp, Uint128};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]

pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid minter code id")]
    InvalidMinterCodeId {},
}
