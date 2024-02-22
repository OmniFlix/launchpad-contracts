use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Decimal, StdError, Timestamp};
use omniflix_std::types::omniflix::onft::v1beta1::{
    Metadata, MsgCreateDenom, MsgMintOnft, WeightedAddress,
};

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
    // FE: Collection:"Badkids" each token name "BadKid" #token_id
    pub royalty_receivers: Option<Vec<WeightedAddress>>,
}
#[cw_serde]
pub struct TokenDetails {
    // FE: Collection:"Badkids" description: "Collection of Badkids", token{ description: "Badkid from badkids collection", name: "Badkid", symbol: "BKID", uri: "https://badkids.com/1", uri_hash: "QmZG9Z3Y9Z3Y}
    pub data: Option<String>,
    pub description: Option<String>,
    pub preview_uri: Option<String>,
    pub token_name: String,
    pub transferable: bool,
    pub extensible: bool,
    pub nsfw: bool,
    pub royalty_ratio: Decimal,
    // This preview_uri is used for the preview of the token. If provided, it will be used as the preview_uri+token_id
    // This is the base token uri. If provided, it will be used as the base_token_uri+token_id should be pointing at a json file.
    pub base_token_uri: String,
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

#[cw_serde]
pub struct AuthDetails {
    pub admin: Addr,
    pub payment_collector: Addr,
}

#[cw_serde]
pub struct MinterInstantiateMsg<T> {
    pub collection_details: CollectionDetails,
    pub token_details: TokenDetails,
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
pub enum QueryMsg<T> {
    #[returns(CollectionDetails)]
    Collection {},
    #[returns(TokenDetails)]
    TokenDetails {},
    #[returns(Config)]
    Config {},
    #[returns(UserDetails)]
    MintedTokens { address: String },
    #[returns(bool)]
    IsPaused {},
    #[returns(Vec<Addr>)]
    Pausers {},
    #[returns(u32)]
    Extension(T),
    #[returns(u32)]
    TotalMintedCount {},
}

pub fn generate_mint_message(
    collection: &CollectionDetails,
    token_details: &TokenDetails,
    token_id: String,
    minter_address: Addr,
    recipient: Addr,
    // Purpose of drop_token_number is to handle the case when the token is an drop
    drop_token_number: Option<String>,
    is_edition: bool,
) -> MsgMintOnft {
    match is_edition {
        false => {
            let metadata = Metadata {
                name: format!("{} # {}", token_details.token_name.clone(), token_id),
                description: token_details.description.clone().unwrap_or("".to_string()),
                media_uri: format!("{}/{}", token_details.base_token_uri.clone(), token_id),
                preview_uri: format!(
                    "{}/{}",
                    token_details
                        .preview_uri
                        .clone()
                        .unwrap_or(token_details.base_token_uri.clone()),
                    token_id,
                ),
                uri_hash: collection.uri_hash.clone().unwrap_or("".to_string()),
            };

            MsgMintOnft {
                data: token_details.data.clone().unwrap_or("".to_string()),
                id: token_id,
                metadata: Some(metadata),
                denom_id: collection.id.clone(),
                transferable: token_details.transferable,
                sender: minter_address.into_string(),
                extensible: token_details.extensible,
                nsfw: token_details.nsfw,
                recipient: recipient.clone().into_string(),
                royalty_share: token_details.royalty_ratio.atomics().to_string(),
            }
        }
        true => {
            let metadata = Metadata {
                name: format!(
                    "{} # {}",
                    token_details.token_name.clone(),
                    drop_token_number.unwrap_or(token_id.clone())
                ),
                description: token_details.description.clone().unwrap_or("".to_string()),
                media_uri: token_details.base_token_uri.clone(),
                preview_uri: token_details
                    .preview_uri
                    .clone()
                    .unwrap_or(token_details.base_token_uri.clone()),
                uri_hash: "".to_string(),
            };

            MsgMintOnft {
                data: token_details.data.clone().unwrap_or("".to_string()),
                id: token_id,
                metadata: Some(metadata),
                denom_id: collection.id.clone(),
                transferable: token_details.transferable,
                sender: minter_address.into_string(),
                extensible: token_details.extensible,
                nsfw: token_details.nsfw,
                recipient: recipient.clone().into_string(),
                royalty_share: token_details.royalty_ratio.atomics().to_string(),
            }
        }
    }
}
pub fn generate_create_denom_msg(
    collection: &CollectionDetails,
    minter_address: Addr,
    creation_fee: Coin,
    admin: Addr,
) -> Result<MsgCreateDenom, StdError> {
    let create_denom_msg = MsgCreateDenom {
        creation_fee: Some(creation_fee.into()),
        id: collection.id.clone(),
        symbol: collection.symbol.clone(),
        name: collection.collection_name.clone(),
        description: collection.description.clone().unwrap_or("".to_string()),
        preview_uri: collection.preview_uri.clone().unwrap_or("".to_string()),
        schema: collection.schema.clone().unwrap_or("".to_string()),
        sender: minter_address.into_string(),
        uri: collection.uri.clone().unwrap_or("".to_string()),
        uri_hash: collection.uri_hash.clone().unwrap_or("".to_string()),
        data: collection.data.clone().unwrap_or("".to_string()),
        royalty_receivers: collection.royalty_receivers.clone().unwrap_or(
            [WeightedAddress {
                address: admin.into_string(),
                weight: Decimal::one().to_string(),
            }]
            .to_vec(),
        ),
    };
    Ok(create_denom_msg)
}
