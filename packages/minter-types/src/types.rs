use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal, Timestamp};
use omniflix_std::types::omniflix::onft::v1beta1::WeightedAddress;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TokenDetailsError {
    #[error("Invalid royalty ratio")]
    InvalidRoyaltyRatio {},
    #[error("Base token uri too long")]
    BaseTokenUriTooLong {},
    #[error("Preview uri too long")]
    PreviewUriTooLong {},
    #[error("Token description too long")]
    TokenDescriptionTooLong {},
    #[error("Token name too long")]
    TokenNameTooLong {},
    #[error("Data too long")]
    DataTooLong {},
}
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
pub struct TokenDetails {
    // Name of each individual token
    // FE: Collection:{collection_name: "Baby Tardigrades", description: "Collection of Baby Tardigrades"},
    // Each Token{token_name: "Baby Tardigrade",description: "Baby Tardigrade from Baby Tardigrades collection"}
    pub token_name: String,
    pub data: Option<String>,
    pub description: Option<String>,
    pub transferable: bool,
    pub extensible: bool,
    pub nsfw: bool,
    pub royalty_ratio: Decimal,
    // Base token uri. It will be used as the base_token_uri+token_id. Its expected to be a json file of token details.
    pub base_token_uri: String,
    // Preview_uri is used for the preview of the token. If provided, it will be used as the preview_uri+token_id
    pub preview_uri: Option<String>,
}
impl TokenDetails {
    pub fn check_integrity(&self) -> Result<(), TokenDetailsError> {
        if self.royalty_ratio < Decimal::zero() || self.royalty_ratio > Decimal::one() {
            return Err(TokenDetailsError::InvalidRoyaltyRatio {});
        }
        if self.base_token_uri.chars().count() > 256 {
            return Err(TokenDetailsError::BaseTokenUriTooLong {});
        }
        if let Some(preview_uri) = &self.preview_uri {
            if preview_uri.chars().count() > 256 {
                return Err(TokenDetailsError::PreviewUriTooLong {});
            }
        }
        if let Some(description) = &self.description {
            if description.chars().count() > 4096 {
                return Err(TokenDetailsError::TokenDescriptionTooLong {});
            }
        }
        if self.token_name.chars().count() > 256 {
            return Err(TokenDetailsError::TokenNameTooLong {});
        }

        if let Some(data) = &self.data {
            if data.chars().count() > 4096 {
                return Err(TokenDetailsError::DataTooLong {});
            }
        }
        Ok(())
    }
}

#[cw_serde]
pub struct CollectionDetails {
    pub description: Option<String>,
    pub preview_uri: Option<String>,
    pub schema: Option<String>,
    pub uri: Option<String>,
    pub uri_hash: Option<String>,
    pub collection_name: String,
    pub data: Option<String>,
    pub symbol: String,
    pub id: String,
    // FE: Collection:"Baby Tardigrades" each token name "Baby Tardigrade" #token_id
    pub royalty_receivers: Option<Vec<WeightedAddress>>,
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

#[cw_serde]
pub struct AuthDetails {
    pub admin: Addr,
    pub payment_collector: Addr,
}

#[derive(Default)]
#[cw_serde]
pub struct UserDetails {
    pub minted_tokens: Vec<Token>,
    pub total_minted_count: u32,
    pub public_mint_count: u32,
}

#[cw_serde]
pub struct Token {
    pub token_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MultiMintData {
    pub token_name: String,
    pub drop_id: String,
    pub drop_token_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NftData {
    pub creator_token_data: String,
    pub multi_mint_data: Option<MultiMintData>,
}
