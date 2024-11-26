use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TokenDetailsError {
    #[error("Invalid royalty ratio")]
    InvalidRoyaltyRatio {},
    #[error("Base token uri too long")]
    BaseTokenUriTooLong {},
    #[error("Base token uri too short")]
    BaseTokenUriTooShort {},
    #[error("Preview uri too long")]
    PreviewUriTooLong {},
    #[error("Preview uri too short")]
    PreviewUriTooShort {},
    #[error("Token description too long")]
    TokenDescriptionTooLong {},
    #[error("Token description too short")]
    TokenDescriptionTooShort {},
    #[error("Token name too long")]
    TokenNameTooLong {},
    #[error("Token name too short")]
    TokenNameTooShort {},
    #[error("Data too long")]
    DataTooLong {},
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
        if self.base_token_uri.chars().count() < 3 {
            return Err(TokenDetailsError::BaseTokenUriTooShort {});
        }
        if let Some(preview_uri) = &self.preview_uri {
            if preview_uri.chars().count() > 256 {
                return Err(TokenDetailsError::PreviewUriTooLong {});
            }
        }
        if let Some(preview_uri) = &self.preview_uri {
            if preview_uri.chars().count() < 3 {
                return Err(TokenDetailsError::PreviewUriTooShort {});
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
        if self.token_name.chars().count() < 3 {
            return Err(TokenDetailsError::TokenNameTooShort {});
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
pub struct Token {
    pub token_id: String,
}

#[cw_serde]
pub struct MultiMintData {
    pub token_name: String,
    pub mint_instance_id: String,
    pub mint_instance_token_id: String,
}
#[cw_serde]
pub struct NftData {
    pub creator_token_data: String,
    pub multi_mint_data: Option<MultiMintData>,
}
