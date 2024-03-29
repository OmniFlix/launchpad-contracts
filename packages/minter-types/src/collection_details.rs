use cosmwasm_schema::cw_serde;
use omniflix_std::types::omniflix::onft::v1beta1::WeightedAddress;
use thiserror::Error;
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

// TODO : Validate values bellow
impl CollectionDetails {
    pub fn check_integrity(&self) -> Result<(), CollectionDetailsError> {
        if self.collection_name.chars().count() > 256 {
            return Err(CollectionDetailsError::InvalidCollectionName {});
        }
        if self.symbol.chars().count() > 256 {
            return Err(CollectionDetailsError::InvalidSymbol {});
        }

        if let Some(description) = &self.description {
            if description.chars().count() > 4096 {
                return Err(CollectionDetailsError::InvalidDescription {});
            }
        }

        if let Some(preview_uri) = &self.preview_uri {
            if preview_uri.chars().count() > 256 {
                return Err(CollectionDetailsError::InvalidPreviewUri {});
            }
        }

        if let Some(schema) = &self.schema {
            if schema.chars().count() > 256 {
                return Err(CollectionDetailsError::InvalidSchema {});
            }
        }

        if let Some(uri) = &self.uri {
            if uri.chars().count() > 256 {
                return Err(CollectionDetailsError::InvalidUri {});
            }
        }

        if let Some(uri_hash) = &self.uri_hash {
            if uri_hash.chars().count() > 256 {
                return Err(CollectionDetailsError::InvalidUriHash {});
            }
        }

        if let Some(data) = &self.data {
            if data.chars().count() > 4096 {
                return Err(CollectionDetailsError::InvalidData {});
            }
        }
        Ok(())
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum CollectionDetailsError {
    #[error("Invalid collection name")]
    InvalidCollectionName {},
    #[error("Invalid symbol")]
    InvalidSymbol {},
    #[error("Invalid description")]
    InvalidDescription {},
    #[error("Invalid preview uri")]
    InvalidPreviewUri {},
    #[error("Invalid schema")]
    InvalidSchema {},
    #[error("Invalid uri")]
    InvalidUri {},
    #[error("Invalid uri hash")]
    InvalidUriHash {},
    #[error("Invalid data")]
    InvalidData {},
}
