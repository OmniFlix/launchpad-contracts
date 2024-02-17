use std::{fmt::format, ptr::NonNull};

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Decimal, StdError, Storage, Timestamp};
use cw_storage_plus::Item;
use omniflix_std::types::omniflix::onft::v1beta1::{Metadata, MsgMintOnft, WeightedAddress};
use thiserror::Error;

#[cw_serde]
pub struct CollectionDetails {
    pub name: String,
    pub description: String,
    pub preview_uri: String,
    pub schema: String,
    pub symbol: String,
    pub id: String,
    pub extensible: bool,
    pub nsfw: bool,
    pub base_uri: String,
    pub uri: String,
    pub uri_hash: Option<String>,
    pub data: String,
    pub transferable: bool,
    // FE: Collection:"Badkids" each token name "BadKid" #token_id
    pub token_name: String,
    pub royalty_receivers: Option<Vec<WeightedAddress>>,
}

#[cw_serde]
pub struct MinterInstantiateMsg<T> {
    pub collection_details: CollectionDetails,
    pub init: T,
}

#[cw_serde]
pub struct UserDetails {
    pub minted_tokens: Vec<Token>,
    pub total_minted_count: u32,
    pub public_mint_count: u32,
}

impl Default for UserDetails {
    fn default() -> Self {
        UserDetails {
            minted_tokens: Vec::new(),
            total_minted_count: 0,
            public_mint_count: 0,
        }
    }
}

#[cw_serde]
pub struct Token {
    pub token_id: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CollectionDetails)]
    Collection {},
    #[returns(Config)]
    Config {},
    #[returns(Vec<Token>)]
    MintableTokens {},
    #[returns(UserDetails)]
    MintedTokens { address: String },
    #[returns(u32)]
    TotalTokens {},
    #[returns(bool)]
    IsPaused {},
    #[returns(Vec<Addr>)]
    Pausers {},
}

#[cw_serde]
pub struct Config {
    pub per_address_limit: u32,
    pub payment_collector: Addr,
    pub start_time: Timestamp,
    pub end_time: Option<Timestamp>,
    pub mint_price: Coin,
    pub royalty_ratio: Decimal,
    pub admin: Addr,
    pub whitelist_address: Option<Addr>,
    pub token_limit: Option<u32>,
}

pub fn generate_mint_message(
    collection: &CollectionDetails,
    royalty_ratio: Decimal,
    recipient: &Addr,
    contract_address: &Addr,
    is_edition: bool,
    token_id: String,
    // Purpose of token_number is to handle the case when the token is an drop
    drop_token_number: Option<String>,
) -> MsgMintOnft {
    match is_edition {
        false => {
            let metadata = Metadata {
                name: format!("{} # {}", collection.token_name.clone(), token_id),
                description: collection.description.clone(),
                media_uri: format!("{}/{}", collection.base_uri, token_id),
                preview_uri: collection.preview_uri.clone(),
                uri_hash: collection.uri_hash.clone().unwrap_or("".to_string()),
            };

            MsgMintOnft {
                data: collection.data.clone(),
                id: token_id,
                metadata: Some(metadata),
                denom_id: collection.id.clone(),
                transferable: collection.transferable,
                sender: contract_address.clone().into_string(),
                extensible: collection.extensible,
                nsfw: collection.nsfw,
                recipient: recipient.clone().into_string(),
                royalty_share: royalty_ratio.atomics().to_string(),
            }
        }
        true => {
            let metadata = Metadata {
                name: format!(
                    "{} # {}",
                    collection.token_name.clone(),
                    drop_token_number.unwrap_or(token_id.clone())
                ),
                description: collection.description.clone(),
                media_uri: collection.base_uri.clone(),
                preview_uri: collection.preview_uri.clone(),
                uri_hash: "".to_string(),
            };

            MsgMintOnft {
                data: collection.data.clone(),
                id: token_id,
                metadata: Some(metadata),
                denom_id: collection.id.clone(),
                transferable: collection.transferable,
                sender: contract_address.clone().into_string(),
                extensible: collection.extensible,
                nsfw: collection.nsfw,
                recipient: recipient.clone().into_string(),
                royalty_share: royalty_ratio.atomics().to_string(),
            }
        }
    }
}
