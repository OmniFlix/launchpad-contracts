use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Decimal, StdError, Storage, Timestamp};
use cw_storage_plus::Item;
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
    pub uri_hash: String,
    pub data: String,
    pub transferable: bool,
    // FE: Collection:"Badkids" each token name "BadKid" #token_id
    pub token_name: String,
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
}

impl Default for UserDetails {
    fn default() -> Self {
        UserDetails {
            minted_tokens: Vec::new(),
            total_minted_count: 0,
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

#[derive(Error, Debug, PartialEq)]
pub enum PauseError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("contract is paused")]
    Paused {},

    #[error("unauthorized pauser ({sender})")]
    Unauthorized { sender: Addr },
}

pub struct PauseState<'a> {
    pub paused: Item<'a, bool>,
    pub pausers: Item<'a, Vec<Addr>>,
}

impl<'a> PauseState<'a> {
    /// Creates a new pause orchestrator using the provided storage
    /// keys.
    pub const fn new(paused_key: &'a str, pausers_key: &'a str) -> Self {
        Self {
            paused: Item::new(paused_key),
            pausers: Item::new(pausers_key),
        }
    }

    /// Sets a new pauser who may pause the contract. If the contract
    /// is paused, it is unpaused.
    pub fn set_pausers(
        &self,
        storage: &mut dyn Storage,
        sender: Addr,
        pausers: Vec<Addr>,
    ) -> Result<(), PauseError> {
        self.error_if_unauthorized(storage, &sender)?;
        self.pausers.save(storage, &pausers)?;
        Ok(())
    }

    /// Errors if the module is paused, does nothing otherwise.
    pub fn error_if_paused(&self, storage: &dyn Storage) -> Result<(), PauseError> {
        if self.paused.load(storage)? {
            Err(PauseError::Paused {})
        } else {
            Ok(())
        }
    }
    pub fn error_if_unauthorized(
        &self,
        storage: &dyn Storage,
        sender: &Addr,
    ) -> Result<(), PauseError> {
        let pausers = self.pausers.load(storage)?;
        if !pausers.contains(sender) {
            Err(PauseError::Unauthorized {
                sender: sender.clone(),
            })
        } else {
            Ok(())
        }
    }

    pub fn pause(&self, storage: &mut dyn Storage, sender: &Addr) -> Result<(), PauseError> {
        self.error_if_paused(storage)?;
        self.error_if_unauthorized(storage, sender)?;
        self.paused.save(storage, &true)?;
        Ok(())
    }

    pub fn unpause(&self, storage: &mut dyn Storage, sender: &Addr) -> Result<(), PauseError> {
        self.error_if_paused(storage)?;
        self.error_if_unauthorized(storage, sender)?;
        self.paused.save(storage, &false)?;
        Ok(())
    }
}
