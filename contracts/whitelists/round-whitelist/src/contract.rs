#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, to_json_binary, wasm_execute, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdResult, Timestamp, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::{maybe_addr, must_pay};

use types::whitelist::{
    Config, HasEndedResponse, HasMemberResponse, HasStartedResponse, IsActiveResponse,
    MembersResponse, PerAddressLimitResponse, WhitelistQueryMsgs,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, "whitelist-round", "1.0.0");

    // TODO: Check instantiator is factory contract

    let admin = maybe_addr(info.sender).unwrap_or(info.sender);

    Ok(Response::default())
}
