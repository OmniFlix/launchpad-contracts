use crate::error::ContractError;
use cosmwasm_std::{Coin, Timestamp};
use whitelist_types::Round;

pub trait RoundMethods {
    fn is_active(&self, current_time: Timestamp) -> bool;
    fn has_started(&self, current_time: Timestamp) -> bool;
    fn has_ended(&self, current_time: Timestamp) -> bool;
    fn mint_price(&self) -> Coin;
    fn check_integrity(&self, now: Timestamp) -> Result<(), ContractError>;
}
impl RoundMethods for Round {
    fn is_active(&self, current_time: Timestamp) -> bool {
        current_time >= self.start_time && current_time <= self.end_time
    }
    fn has_started(&self, current_time: Timestamp) -> bool {
        current_time >= self.start_time
    }
    fn has_ended(&self, current_time: Timestamp) -> bool {
        current_time > self.end_time
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
}
