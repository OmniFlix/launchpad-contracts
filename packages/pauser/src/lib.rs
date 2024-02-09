use std::fmt::format;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Decimal, StdError, Storage, Timestamp};
use cw_storage_plus::Item;
use thiserror::Error;
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
    pub fn new(paused_key: &'a str, pausers_key: &'a str) -> Result<Self, PauseError> {
        let paused = Item::new(paused_key);
        let pausers = Item::new(pausers_key);
        Ok(PauseState { paused, pausers })
    }

    /// Sets a new pauser who may pause the contract.
    /// If no pausers are set, sets pausers to the provided addresses without authorization.
    /// If pausers are already set, sender must be one of the pausers.
    /// Also unpauses
    pub fn set_pausers(
        &self,
        storage: &mut dyn Storage,
        sender: Addr,
        pausers: Vec<Addr>,
    ) -> Result<(), PauseError> {
        let mut current_pausers = self.pausers.load(storage).unwrap_or(vec![]);
        if current_pausers.is_empty() {
            current_pausers = pausers;
        } else {
            self.error_if_unauthorized(storage, &sender)?;
            current_pausers = pausers;
        }
        self.pausers.save(storage, &current_pausers)?;
        self.paused.save(storage, &false)?;
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
        self.error_if_unauthorized(storage, sender)?;
        self.paused.save(storage, &false)?;
        Ok(())
    }

    pub fn is_paused(&self, storage: &dyn Storage) -> Result<bool, PauseError> {
        let is_paused = self.paused.load(storage).unwrap_or(false);
        Ok(is_paused)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;

    #[test]
    fn test_pause_state() {
        let mut deps = mock_dependencies();

        let pauser1 = Addr::unchecked("pauser1");
        let pauser2 = Addr::unchecked("pauser2");
        let pauser3 = Addr::unchecked("pauser3");

        let state = PauseState::new("paused", "pausers").unwrap();

        // no pausers set
        assert_eq!(
            state.set_pausers(&mut deps.storage, pauser1.clone(), vec![]),
            Ok(())
        );
        assert_eq!(
            state.set_pausers(&mut deps.storage, pauser2.clone(), vec![]),
            Ok(())
        );
        assert_eq!(
            state.set_pausers(&mut deps.storage, pauser3.clone(), vec![]),
            Ok(())
        );

        // pausers set
        assert_eq!(
            state.set_pausers(
                &mut deps.storage,
                pauser1.clone(),
                vec![pauser1.clone(), pauser2.clone()]
            ),
            Ok(())
        );

        assert_eq!(
            state.set_pausers(
                &mut deps.storage,
                pauser2.clone(),
                vec![pauser1.clone(), pauser2.clone()]
            ),
            Ok(())
        );
        assert_eq!(
            state.set_pausers(
                &mut deps.storage,
                pauser3.clone(),
                vec![pauser1.clone(), pauser2.clone()]
            ),
            Err(PauseError::Unauthorized {
                sender: pauser3.clone()
            })
        );

        // pause
        assert_eq!(state.pause(&mut deps.storage, &pauser1), Ok(()));
        assert_eq!(
            state.pause(&mut deps.storage, &pauser2),
            Err(PauseError::Paused {})
        );
        assert_eq!(
            state.pause(&mut deps.storage, &pauser3.clone()),
            Err(PauseError::Paused {})
        );

        // unpause
        assert_eq!(state.unpause(&mut deps.storage, &pauser1), Ok(()));
        assert_eq!(state.unpause(&mut deps.storage, &pauser2), Ok(()));
        assert_eq!(
            state.unpause(&mut deps.storage, &pauser3),
            Err(PauseError::Unauthorized { sender: pauser3 })
        );
    }
}
