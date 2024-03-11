use cosmwasm_std::{Addr, Coin, Decimal, QuerierWrapper, StdError, Uint128};
use omniflix_std::types::omniflix::onft::v1beta1::{
    Metadata, MsgCreateDenom, MsgMintOnft, MsgUpdateDenom, OnftQuerier, WeightedAddress,
};
use std::str::FromStr;

use crate::types::{CollectionDetails, TokenDetails};

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
                name: format!("{} #{}", token_details.token_name.clone(), token_id),
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
                    "{} #{}",
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
    payment_collector: Addr,
) -> Result<MsgCreateDenom, StdError> {
    let royalty_receivers = match &collection.royalty_receivers {
        Some(receivers) => receivers
            .iter()
            .map(|r| {
                let atomics_weight = Decimal::from_str(&r.weight)?.atomics().to_string();
                Ok(WeightedAddress {
                    address: r.address.clone(),
                    weight: atomics_weight,
                })
            })
            .collect::<Result<Vec<WeightedAddress>, StdError>>()?,

        None => vec![WeightedAddress {
            address: payment_collector.into_string(),
            weight: Decimal::one().atomics().to_string(),
        }],
    };

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
        royalty_receivers: royalty_receivers,
    };
    Ok(create_denom_msg)
}
pub fn generate_update_denom_msg(
    collection: &CollectionDetails,
    payment_collector: Addr,
    minter_address: Addr,
) -> Result<MsgUpdateDenom, StdError> {
    let atomics_royalty_receivers = match &collection.royalty_receivers {
        Some(receivers) => receivers
            .iter()
            .map(|r| {
                let atomics_weight = Decimal::from_str(&r.weight)?.atomics().to_string();
                Ok(WeightedAddress {
                    address: r.address.clone(),
                    weight: atomics_weight,
                })
            })
            .collect::<Result<Vec<WeightedAddress>, StdError>>()?,
        None => vec![WeightedAddress {
            address: payment_collector.into_string(),
            weight: Decimal::one().atomics().to_string(),
        }],
    };
    let update_denom_msg = MsgUpdateDenom {
        id: collection.id.clone(),
        sender: minter_address.into_string(),
        name: collection.collection_name.clone(),
        description: collection
            .description
            .clone()
            .unwrap_or("[do-not-modify]".to_string()),
        preview_uri: collection
            .preview_uri
            .clone()
            .unwrap_or("[do-not-modify]".to_string()),
        royalty_receivers: atomics_royalty_receivers,
    };
    Ok(update_denom_msg)
}

pub fn update_collection_details(
    collection: &CollectionDetails,
    collection_name: Option<String>,
    description: Option<String>,
    preview_uri: Option<String>,
    royalty_receivers: Option<Vec<WeightedAddress>>,
) -> CollectionDetails {
    let mut new_collection = collection.clone();
    if let Some(name) = collection_name {
        new_collection.collection_name = name;
    }
    if let Some(desc) = description {
        new_collection.description = Some(desc);
    }
    if let Some(preview) = preview_uri {
        new_collection.preview_uri = Some(preview);
    }
    if let Some(receivers) = royalty_receivers {
        new_collection.royalty_receivers = Some(receivers);
    }
    new_collection
}

pub fn check_collection_creation_fee(querier: QuerierWrapper) -> Result<Coin, StdError> {
    // If creation fee is 0, then it means this is not a test case
    let onft_querier = OnftQuerier::new(&querier);
    let params = onft_querier.params()?;
    let creation_fee = params.params.unwrap().denom_creation_fee.unwrap();
    Ok(Coin {
        denom: creation_fee.denom,
        amount: Uint128::from_str(&creation_fee.amount)?,
    })
}
