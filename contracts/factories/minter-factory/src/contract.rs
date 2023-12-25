use crate::error::ContractError;
use crate::msg::InstantiateMsg;
use crate::state::{Params, PARAMS};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdResult, Timestamp, Uint128, WasmMsg,
};
use cw_utils::{maybe_addr, must_pay, nonpayable};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let admin = maybe_addr(deps.api, msg.admin)?.unwrap_or(info.sender);
    let fee_collector_address = deps.api.addr_validate(&msg.fee_collector_address)?;
    if msg.minter_code_id == 0 {
        return Err(ContractError::InvalidMinterCodeId {});
    }
    let params = Params {
        admin: admin.clone(),
        allowed_minter_mint_denoms: msg.allowed_minter_mint_denoms,
        fee_collector_address: fee_collector_address,
        minter_code_id: msg.minter_code_id,
        minter_creation_fee: msg.minter_creation_fee,
    };
    PARAMS.save(deps.storage, &params)?;
    Ok(Response::default())
}
