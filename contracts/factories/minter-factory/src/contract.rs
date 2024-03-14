use crate::error::ContractError;
use crate::msg::{CreateMinterMsg, ExecuteMsg, InstantiateMsg, ParamsResponse, QueryMsg};
use crate::state::PARAMS;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128, WasmMsg,
};
use factory_types::check_payment;
use minter_types::utils::check_collection_creation_fee;
use pauser::PauseState;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let admin = deps
        .api
        .addr_validate(&msg.params.clone().admin.into_string())
        .unwrap_or(info.sender.clone());
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
        ExecuteMsg::CreateMinter { msg } => create_minter(deps, env, info, msg),
        ExecuteMsg::UpdateAdmin { admin } => update_params_admin(deps, env, info, admin),
        ExecuteMsg::UpdateFeeCollectorAddress {
            fee_collector_address,
        } => update_params_fee_collector_address(deps, env, info, fee_collector_address),
        ExecuteMsg::UpdateMinterCodeId { minter_code_id } => {
            update_params_minter_code_id(deps, env, info, minter_code_id)
        }
        ExecuteMsg::UpdateMinterCreationFee {
            minter_creation_fee,
        } => update_params_minter_creation_fee(deps, env, info, minter_creation_fee),
        ExecuteMsg::Pause {} => execute_pause(deps, env, info),
        ExecuteMsg::Unpause {} => execute_unpause(deps, env, info),
        ExecuteMsg::SetPausers { pausers } => set_pausers(deps, env, info, pausers),
    }
}

fn create_minter(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: CreateMinterMsg,
) -> Result<Response, ContractError> {
    let params = PARAMS.load(deps.storage)?;
    let collection_creation_fee: Coin = check_collection_creation_fee(deps.as_ref().querier)?;
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.as_ref().storage)?;

    check_payment(
        &info.funds,
        &[
            collection_creation_fee.clone(),
            params.minter_creation_fee.clone(),
        ],
    )?;
    let mut msgs = Vec::<CosmosMsg>::new();
    msgs.push(CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(msg.init.admin.to_string()),
        code_id: params.minter_code_id,
        msg: to_json_binary(&msg)?,
        funds: vec![collection_creation_fee.clone()],
        label: params.product_label,
    }));
    if params.minter_creation_fee.amount > Uint128::new(0) {
        msgs.push(CosmosMsg::Bank(BankMsg::Send {
            amount: vec![params.minter_creation_fee.clone()],
            to_address: params.fee_collector_address.to_string(),
        }));
    }
    let res = Response::new()
        .add_messages(msgs)
        .add_attribute("action", "create_minter");
    Ok(res)
}

fn update_params_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: String,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;
    if params.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    params.admin = deps.api.addr_validate(&admin)?;
    PARAMS.save(deps.storage, &params)?;
    Ok(Response::default()
        .add_attribute("action", "update_admin")
        .add_attribute("new_admin", admin))
}

fn update_params_fee_collector_address(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    fee_collector_address: String,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;
    if params.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    params.fee_collector_address = deps.api.addr_validate(&fee_collector_address)?;
    PARAMS.save(deps.storage, &params)?;
    Ok(Response::default()
        .add_attribute("action", "update_fee_collector_address")
        .add_attribute(
            "new_fee_collector_address",
            fee_collector_address.to_string(),
        ))
}

fn update_params_minter_code_id(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    minter_code_id: u64,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;
    if params.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    params.minter_code_id = minter_code_id;
    PARAMS.save(deps.storage, &params)?;
    Ok(Response::default()
        .add_attribute("action", "update_minter_code_id")
        .add_attribute("new_minter_code_id", minter_code_id.to_string()))
}

fn update_params_minter_creation_fee(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    minter_creation_fee: Coin,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;
    if params.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    params.minter_creation_fee = minter_creation_fee.clone();
    PARAMS.save(deps.storage, &params)?;
    Ok(Response::default()
        .add_attribute("action", "update_minter_creation_fee")
        .add_attribute("new_minter_creation_fee", minter_creation_fee.to_string()))
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
        QueryMsg::IsPaused {} => to_json_binary(&query_is_paused(deps, _env)?),
        QueryMsg::Pausers {} => to_json_binary(&query_pausers(deps, _env)?),
    }
}

fn query_params(deps: Deps) -> StdResult<ParamsResponse> {
    let params = PARAMS.load(deps.storage)?;
    Ok(ParamsResponse { params })
}
fn query_is_paused(deps: Deps, _env: Env) -> Result<bool, ContractError> {
    let pause_state = PauseState::new()?;
    let is_paused = pause_state.is_paused(deps.storage)?;
    Ok(is_paused)
}

fn query_pausers(deps: Deps, _env: Env) -> Result<Vec<Addr>, ContractError> {
    let pause_state = PauseState::new()?;
    let pausers = pause_state.pausers.load(deps.storage).unwrap_or(vec![]);
    Ok(pausers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr,
    };

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            params: crate::msg::MinterFactoryParams {
                admin: Addr::unchecked("admin"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                minter_code_id: 1,
                minter_creation_fee: Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
                product_label: "omniflix-nft-minter".to_string(),
            },
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query params
        let params = query_params(deps.as_ref()).unwrap();
        assert_eq!(
            params.params,
            crate::msg::MinterFactoryParams {
                admin: Addr::unchecked("admin"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                minter_code_id: 1,
                minter_creation_fee: Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
                product_label: "omniflix-nft-minter".to_string(),
            }
        );
    }
}
