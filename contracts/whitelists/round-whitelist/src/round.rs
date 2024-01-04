use cosmwasm_std::{Addr, Coin, Deps, Timestamp};
use whitelist_types::Round;

use crate::error::ContractError;
pub trait RoundMethods {
    fn is_active(&self, current_time: Timestamp) -> bool;
    fn is_member(&self, address: &Addr) -> bool;
    fn has_started(&self, current_time: Timestamp) -> bool;
    fn has_ended(&self, current_time: Timestamp) -> bool;
    fn members(
        &self,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> Result<Vec<String>, ContractError>;
    fn mint_price(&self) -> Coin;
    fn check_integrity(&self, deps: Deps, now: Timestamp) -> Result<(), ContractError>;
}
impl RoundMethods for Round {
    fn is_active(&self, current_time: Timestamp) -> bool {
        current_time >= self.start_time && current_time <= self.end_time
    }

    fn is_member(&self, address: &Addr) -> bool {
        self.addresses.contains(address)
    }

    fn has_started(&self, current_time: Timestamp) -> bool {
        current_time >= self.start_time
    }

    fn has_ended(&self, current_time: Timestamp) -> bool {
        current_time > self.end_time
    }
    fn members(
        &self,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> Result<Vec<String>, ContractError> {
        let mut members: Vec<String> = self.addresses.iter().map(|x| x.to_string()).collect();
        let start_after = start_after.unwrap_or_default();
        let start_index = members
            .iter()
            .position(|x| x.as_str() == start_after.as_str())
            .unwrap_or_default();
        let end_index = match limit {
            Some(limit) => start_index + limit as usize,
            None => members.len(),
        };
        Ok(members[start_index..end_index].to_vec())
    }

    fn mint_price(&self) -> Coin {
        self.mint_price.clone()
    }
    fn check_integrity(&self, deps: Deps, now: Timestamp) -> Result<(), ContractError> {
        if self.start_time > self.end_time {
            return Err(ContractError::InvalidStartTime {});
        }
        if self.start_time < now {
            return Err(ContractError::RoundAlreadyStarted {});
        }
        if self.round_per_address_limit == 0 {
            return Err(ContractError::InvalidPerAddressLimit {});
        }
        if self.addresses.is_empty() {
            return Err(ContractError::EmptyAddressList {});
        }
        self.addresses
            .iter()
            .try_for_each(|address| deps.api.addr_validate(address.as_str()).map(|_| ()))?;

        Ok(())
    }
}
