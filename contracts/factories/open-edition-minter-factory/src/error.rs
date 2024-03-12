use cosmwasm_std::StdError;
use factory_types::CustomPaymentError;
use pauser::PauseError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]

pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Payment error")]
    PaymentError(#[from] CustomPaymentError),

    #[error(transparent)]
    Pause(#[from] PauseError),

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

    #[error("MultiMinter not enabled")]
    MultiMinterNotEnabled {},
}
impl From<ContractError> for StdError {
    fn from(err: ContractError) -> StdError {
        StdError::generic_err(err.to_string())
    }
}
