use crate::error::ContractError;
use crate::state::RoundMethods;
use cosmwasm_std::Timestamp;
use types::Round;

pub fn check_round_overlaps(rounds: Vec<Round>) -> Result<(), ContractError> {
    let mut rounds = rounds;
    rounds.sort_by_key(|round| round.start_time());

    for i in 0..rounds.len() - 1 {
        let current_round = &rounds[i];
        let next_round = &rounds[i + 1];

        if current_round.end_time() > next_round.start_time() {
            return Err(ContractError::InvalidRoundTime {
                round: current_round.clone(),
            });
        }
    }

    Ok(())
}
