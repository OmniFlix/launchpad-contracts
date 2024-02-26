use cosmwasm_std::StdError;
use factory_types::CustomPaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]

pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Payment error")]
    PaymentError(#[from] CustomPaymentError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid minter code id")]
    InvalidMinterCodeId {},

    #[error("Invalid Mint Denom")]
    InvalidMintDenom {},

    #[error("Mint denom not allowed")]
    MintDenomNotAllowed {},

    #[error("Missing creation fee")]
    MissingCreationFee {},

    #[error("Missing minter creation fee")]
    MissingMinterCreationFee {},
}
