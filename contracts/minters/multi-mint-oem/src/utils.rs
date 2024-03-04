use cosmwasm_std::Deps;

use crate::{
    error::ContractError,
    state::{DropParams, CURRENT_DROP_ID, DROPS},
};

pub fn get_drop(drop_id: Option<u32>, deps: Deps) -> Result<DropParams, ContractError> {
    let drop_id = drop_id.unwrap_or(CURRENT_DROP_ID.load(deps.storage)?);
    if drop_id == 0 {
        return Err(ContractError::NoDropAvailable {});
    }
    let drop_params = DROPS
        .load(deps.storage, drop_id)
        .map_err(|_| ContractError::InvalidDropId {})?;
    Ok(drop_params)
}
