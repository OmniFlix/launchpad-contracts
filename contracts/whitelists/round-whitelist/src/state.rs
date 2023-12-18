use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Deps, Timestamp};

use crate::error::ContractError;

#[cw_serde]
pub enum Round {
    WhitelistAddresses {
        addresses: Vec<Addr>,
        start_time: Timestamp,
        end_time: Timestamp,
        mint_price: Coin,
        round_per_address_limit: u32,
    },
    WhitelistCollection {
        collection_id: String,
        start_time: Timestamp,
        end_time: Timestamp,
        mint_price: Coin,
        round_per_address_limit: u32,
    },
}

impl Round {
    pub fn is_active(&self, current_time: Timestamp) -> bool {
        match self {
            Round::WhitelistAddresses {
                start_time,
                end_time,
                ..
            } => current_time >= *start_time && current_time <= *end_time,
            Round::WhitelistCollection {
                start_time,
                end_time,
                ..
            } => current_time >= *start_time && current_time <= *end_time,
        }
    }

    pub fn has_started(&self, current_time: Timestamp) -> bool {
        match self {
            Round::WhitelistAddresses { start_time, .. } => current_time >= *start_time,
            Round::WhitelistCollection { start_time, .. } => current_time >= *start_time,
        }
    }

    pub fn has_ended(&self, current_time: Timestamp) -> bool {
        match self {
            Round::WhitelistAddresses { end_time, .. } => current_time >= *end_time,
            Round::WhitelistCollection { end_time, .. } => current_time >= *end_time,
        }
    }

    pub fn start_time(&self) -> Timestamp {
        match self {
            Round::WhitelistAddresses { start_time, .. } => *start_time,
            Round::WhitelistCollection { start_time, .. } => *start_time,
        }
    }

    pub fn end_time(&self) -> Timestamp {
        match self {
            Round::WhitelistAddresses { end_time, .. } => *end_time,
            Round::WhitelistCollection { end_time, .. } => *end_time,
        }
    }

    pub fn check_integrity(&self, deps: Deps) -> Result<(), ContractError> {
        match self {
            Round::WhitelistAddresses {
                addresses,
                start_time,
                end_time,
                mint_price,
                round_per_address_limit,
            } => {
                if addresses.is_empty() {
                    return Err(ContractError::EmptyAddressList {});
                }
                if *start_time >= *end_time {
                    return Err(ContractError::InvalidStartTime {});
                }
                if *round_per_address_limit == 0 {
                    return Err(ContractError::InvalidPerAddressLimit {});
                }
                for address in addresses {
                    deps.api.addr_validate(address.as_str())?;
                }
            }
            Round::WhitelistCollection {
                collection_id,
                start_time,
                end_time,
                mint_price,
                round_per_address_limit,
            } => {
                // TODO: Validate collection id by Querying the collection
                if collection_id.is_empty() {
                    return Err(ContractError::InvalidMemberLimit {});
                }
                if *start_time >= *end_time {
                    return Err(ContractError::InvalidStartTime {});
                }
                if *round_per_address_limit == 0 {
                    return Err(ContractError::InvalidPerAddressLimit {});
                }
            }
        }
        Ok(())
    }
}
