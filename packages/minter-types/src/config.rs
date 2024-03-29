use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Timestamp};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ConfigurationError {
    #[error("Invalid start time")]
    InvalidStartTime {},
    #[error("Invalid end time")]
    InvalidEndTime {},
    #[error("Invalid per address limit")]
    InvalidPerAddressLimit {},
    #[error("Invalid mint price")]
    InvalidMintPrice {},
    #[error("Invalid whitelist address")]
    InvalidWhitelistAddress {},
    #[error("Invalid number of tokens")]
    InvalidNumberOfTokens {},
}

#[cw_serde]
pub struct Config {
    pub per_address_limit: Option<u32>,
    pub start_time: Timestamp,
    pub end_time: Option<Timestamp>,
    pub whitelist_address: Option<Addr>,
    pub num_tokens: Option<u32>,
    pub mint_price: Coin,
}

impl Config {
    pub fn check_integrity(&self, now: Timestamp) -> Result<(), ConfigurationError> {
        if let Some(per_address_limit) = self.per_address_limit {
            if per_address_limit == 0 {
                return Err(ConfigurationError::InvalidPerAddressLimit {});
            }
        }
        if self.num_tokens == Some(0) {
            return Err(ConfigurationError::InvalidNumberOfTokens {});
        }
        if self.start_time < now {
            return Err(ConfigurationError::InvalidStartTime {});
        }
        if let Some(end_time) = self.end_time {
            if end_time < self.start_time {
                return Err(ConfigurationError::InvalidEndTime {});
            }
        }
        Ok(())
    }
}
