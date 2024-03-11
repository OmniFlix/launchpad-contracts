use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, ParamsResponse, QueryMsg};
use crate::state::PARAMS;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, WasmMsg,
};
use cw_utils::may_pay;
use pauser::PauseState;
use whitelist_types::InstantiateMsg as WhitelistInstantiateMsg;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let admin = deps
        .api
        .addr_validate(&msg.params.clone().admin.into_string())?;
    let _fee_collector_address = deps
        .api
        .addr_validate(&msg.params.fee_collector_address.clone().into_string())
        .unwrap_or(info.sender.clone());
    let pause_state = PauseState::new()?;
    pause_state.set_pausers(deps.storage, info.sender.clone(), vec![admin.clone()])?;
    let params = msg.params;
    PARAMS.save(deps.storage, &params)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateWhitelist { msg } => create_whitelist(deps, env, info, msg),
        ExecuteMsg::UpdateAdmin { admin } => update_admin(deps, env, info, admin),
        ExecuteMsg::UpdateFeeCollectorAddress {
            fee_collector_address,
        } => update_fee_collector_address(deps, env, info, fee_collector_address),
        ExecuteMsg::UpdateWhitelistCreationFee {
            whitelist_creation_fee,
        } => update_whitelist_creation_fee(deps, env, info, whitelist_creation_fee),
        ExecuteMsg::UpdateWhitelistCodeId { whitelist_code_id } => {
            update_whitelist_code_id(deps, env, info, whitelist_code_id)
        }
        ExecuteMsg::Pause {} => execute_pause(deps, env, info),
        ExecuteMsg::Unpause {} => execute_unpause(deps, env, info),
        ExecuteMsg::SetPausers { pausers } => set_pausers(deps, env, info, pausers),
    }
}

pub fn create_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: WhitelistInstantiateMsg,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.as_ref().storage)?;
    let params = PARAMS.load(deps.storage)?;
    let creation_fee = params.whitelist_creation_fee;
    let fee_collector_address = params.fee_collector_address;
    let whitelist_code_id = params.whitelist_code_id;
    let mut messages: Vec<CosmosMsg> = vec![];

    let amount = may_pay(&info, &creation_fee.clone().denom)?;

    if amount != creation_fee.amount {
        return Err(ContractError::MissingCreationFee {});
    }
    if !creation_fee.amount.is_zero() {
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: fee_collector_address.to_string(),
            amount: vec![creation_fee],
        }));
    }
    messages.push(CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(msg.admin.clone()),
        code_id: whitelist_code_id,
        msg: to_json_binary(&msg)?,
        funds: vec![],
        label: params.product_label,
    }));
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "create_whitelist")
        .add_attribute("creator", info.sender))
}

pub fn update_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: String,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;
    if info.sender != params.admin {
        return Err(ContractError::Unauthorized {});
    }
    params.admin = deps.api.addr_validate(&admin)?;
    PARAMS.save(deps.storage, &params)?;
    Ok(Response::new()
        .add_attribute("action", "update_admin")
        .add_attribute("admin", admin))
}

pub fn update_fee_collector_address(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    fee_collector_address: String,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;
    if info.sender != params.admin {
        return Err(ContractError::Unauthorized {});
    }
    params.fee_collector_address = deps.api.addr_validate(&fee_collector_address)?;
    PARAMS.save(deps.storage, &params)?;
    Ok(Response::new().add_attribute("action", "update_fee_collector_address"))
}

pub fn update_whitelist_creation_fee(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    whitelist_creation_fee: Coin,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;
    if info.sender != params.admin {
        return Err(ContractError::Unauthorized {});
    }
    params.whitelist_creation_fee = whitelist_creation_fee;
    PARAMS.save(deps.storage, &params)?;
    Ok(Response::new().add_attribute("action", "update_whitelist_creation_fee"))
}

pub fn update_whitelist_code_id(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    whitelist_code_id: u64,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;
    if info.sender != params.admin {
        return Err(ContractError::Unauthorized {});
    }
    params.whitelist_code_id = whitelist_code_id;
    PARAMS.save(deps.storage, &params)?;
    Ok(Response::new().add_attribute("action", "update_whitelist_code_id"))
}
fn execute_pause(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.pause(deps.storage, &info.sender)?;
    Ok(Response::default()
        .add_attribute("action", "pause")
        .add_attribute("pauser", info.sender))
}

fn execute_unpause(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.unpause(deps.storage, &info.sender)?;
    Ok(Response::default()
        .add_attribute("action", "unpause")
        .add_attribute("pauser", info.sender))
}

fn set_pausers(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    pausers: Vec<String>,
) -> Result<Response, ContractError> {
    let validated_pausers = pausers
        .iter()
        .map(|pauser| deps.api.addr_validate(pauser))
        .collect::<Result<Vec<_>, _>>()?;

    let pause_state = PauseState::new()?;
    pause_state.set_pausers(deps.storage, info.sender.clone(), validated_pausers)?;
    Ok(Response::default()
        .add_attribute("action", "set_pausers")
        .add_attribute("pausers", pausers.join(",")))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Params {} => to_json_binary(&query_params(deps)?),
    }
}

fn query_params(deps: Deps) -> StdResult<ParamsResponse> {
    let params = PARAMS.load(deps.storage)?;
    Ok(ParamsResponse { params })
}
