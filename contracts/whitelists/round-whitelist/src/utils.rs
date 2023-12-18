use cosmwasm_std::Timestamp;

use crate::{error::ContractError, state::Round};

pub fn check_round_overlaps(
    now: Timestamp,
    rounds: Vec<(u32, Round)>,
) -> Result<(), ContractError> {
    let mut rounds = rounds;

    // Sort rounds by start time
    rounds.sort_by(|a, b| a.1.start_time().cmp(&b.1.start_time()));
    // Check for overlaps
    for (i, round) in rounds.iter().enumerate() {
        if i == rounds.len() - 1 {
            break;
        }
        // Check for start time can not be bigger than end time
        if round.1.start_time() > round.1.end_time() {
            return Err(ContractError::InvalidRoundTime {
                round: round.1.clone(),
            });
        }
        let next_round = &rounds[i + 1];
        if round.1.end_time() > next_round.1.start_time() {
            return Err(ContractError::RoundsOverlaped {
                round: round.1.clone(),
            });
        }
    }
    // Check for overlaps with now none of them should be started
    for round in rounds {
        if round.1.start_time() < now {
            return Err(ContractError::RoundAlreadyStarted {});
        }
    }
    Ok(())
}
