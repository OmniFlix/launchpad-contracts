use crate::error::ContractError;
use cosmwasm_std::{Addr, Coin, Deps, Timestamp};
use whitelist_types::Round;

const MEMBER_QUERY_LIMIT: u32 = 100;
const MAX_MEMBERS_PER_ROUND: usize = 5000;
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
    fn check_integrity(&self, now: Timestamp) -> Result<(), ContractError>;
    fn validate_members_and_return(&self, deps: Deps) -> Result<Round, ContractError>;
    fn add_members(&mut self, deps: Deps, address: Vec<String>) -> Result<(), ContractError>;
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
        let start_after = start_after.unwrap_or_default();
        let limit = limit.unwrap_or(MEMBER_QUERY_LIMIT);
        let start_index = self
            .addresses
            .iter()
            .position(|x| x.to_string() == start_after)
            .unwrap_or(0);
        let end_index = (start_index + limit as usize).min(self.addresses.len());
        Ok(self.addresses[start_index..end_index]
            .to_vec()
            .iter()
            .map(|x| x.to_string())
            .collect())
    }

    fn mint_price(&self) -> Coin {
        self.mint_price.clone()
    }
    fn check_integrity(&self, now: Timestamp) -> Result<(), ContractError> {
        if self.start_time > self.end_time {
            return Err(ContractError::InvalidEndTime {});
        }
        if self.start_time < now {
            return Err(ContractError::RoundAlreadyStarted {});
        }
        if self.round_per_address_limit == 0 {
            return Err(ContractError::InvalidPerAddressLimit {});
        }

        Ok(())
    }
    fn validate_members_and_return(&self, deps: Deps) -> Result<Round, ContractError> {
        if self.addresses.is_empty() {
            return Err(ContractError::EmptyAddressList {});
        }
        let mut valid_members: Vec<Addr> = vec![];
        for address in self.addresses.iter() {
            let addr = deps.api.addr_validate(address.as_str())?;
            valid_members.push(addr);
        }
        valid_members.sort();
        valid_members.dedup();
        if valid_members.len() > MAX_MEMBERS_PER_ROUND {
            return Err(ContractError::WhitelistMemberLimitExceeded {});
        }
        Ok(Round {
            addresses: valid_members,
            start_time: self.start_time,
            end_time: self.end_time,
            mint_price: self.mint_price.clone(),
            round_per_address_limit: self.round_per_address_limit,
        })
    }

    fn add_members(&mut self, deps: Deps, address: Vec<String>) -> Result<(), ContractError> {
        let addr_list: Vec<Addr> = address
            .iter()
            .map(|x| deps.api.addr_validate(x.as_str()))
            .collect::<Result<Vec<Addr>, _>>()?;

        self.addresses.extend(addr_list);
        // Remove duplicates final list
        self.addresses.dedup();
        if self.addresses.len() > MAX_MEMBERS_PER_ROUND {
            return Err(ContractError::WhitelistMemberLimitExceeded {});
        }
        Ok(())
    }
}
