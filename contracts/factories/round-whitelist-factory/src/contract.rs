use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, ParamsResponse, QueryMsg};
use crate::state::PARAMS;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, WasmMsg,
};
use cw_utils::may_pay;
use pauser::PauseState;
use whitelist_types::CreateWhitelistMsg;

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
    msg: CreateWhitelistMsg,
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
mod round_whitelist_factory_tests {
    use crate::msg::RoundWhitelistFactoryParams;

    use super::*;
    use cosmwasm_std::{
        from_json,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr,
    };
    use pauser::PauseError;

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let msg = InstantiateMsg {
            params: RoundWhitelistFactoryParams {
                admin: Addr::unchecked("admin"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                whitelist_creation_fee: Coin::new(100, "uflix"),
                whitelist_code_id: 1,
                product_label: "product_label".to_string(),
            },
        };
        let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        // Query params
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Params {}).unwrap();
        let params: ParamsResponse = from_json(res).unwrap();
        assert_eq!(params.params.admin, Addr::unchecked("admin"));
        assert_eq!(
            params.params.fee_collector_address,
            Addr::unchecked("fee_collector_address")
        );
        assert_eq!(
            params.params.whitelist_creation_fee,
            Coin::new(100, "uflix")
        );
        assert_eq!(params.params.whitelist_code_id, 1);
        assert_eq!(params.params.product_label, "product_label");
    }

    #[test]
    fn test_execute_pause_unpause() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let msg = InstantiateMsg {
            params: RoundWhitelistFactoryParams {
                admin: Addr::unchecked("admin"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                whitelist_creation_fee: Coin::new(100, "uflix"),
                whitelist_code_id: 1,
                product_label: "product_label".to_string(),
            },
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Non pauser can not pause
        let info = mock_info("anyone", &[]);
        let res = execute_pause(deps.as_mut(), env.clone(), info);

        assert_eq!(
            res.err().unwrap(),
            ContractError::Pause(PauseError::Unauthorized {
                sender: Addr::unchecked("anyone")
            })
        );

        // Pauser can pause
        let info = mock_info("admin", &[]);
        let _res = execute_pause(deps.as_mut(), env.clone(), info).unwrap();

        // Query is_paused
        let res = query(deps.as_ref(), mock_env(), QueryMsg::IsPaused {}).unwrap();
        let is_paused: bool = from_json(res).unwrap();
        assert!(is_paused);

        // Non pauser can not unpause
        let info = mock_info("anyone", &[]);
        let res = execute_unpause(deps.as_mut(), env.clone(), info);

        assert_eq!(
            res.err().unwrap(),
            ContractError::Pause(PauseError::Unauthorized {
                sender: Addr::unchecked("anyone")
            })
        );

        // Pauser can unpause
        let info = mock_info("admin", &[]);
        let _res = execute_unpause(deps.as_mut(), env.clone(), info).unwrap();

        // Query is_paused
        let res = query(deps.as_ref(), mock_env(), QueryMsg::IsPaused {}).unwrap();
        let is_paused: bool = from_json(res).unwrap();
        assert!(!is_paused);
    }

    #[test]
    fn test_set_pausers() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let msg = InstantiateMsg {
            params: RoundWhitelistFactoryParams {
                admin: Addr::unchecked("admin"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                whitelist_creation_fee: Coin::new(100, "uflix"),
                whitelist_code_id: 1,
                product_label: "product_label".to_string(),
            },
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Non admin can not set pausers
        let info = mock_info("anyone", &[]);
        let res = set_pausers(deps.as_mut(), env.clone(), info, vec!["anyone".to_string()]);

        assert_eq!(
            res.err().unwrap(),
            ContractError::Pause(PauseError::Unauthorized {
                sender: Addr::unchecked("anyone")
            })
        );

        // Admin can set pausers
        let info = mock_info("admin", &[]);
        let _res =
            set_pausers(deps.as_mut(), env.clone(), info, vec!["anyone".to_string()]).unwrap();

        // Query pausers
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Pausers {}).unwrap();
        let pausers: Vec<Addr> = from_json(res).unwrap();
        assert_eq!(pausers.len(), 1);
        assert_eq!(pausers[0], Addr::unchecked("anyone"));
    }

    #[test]
    fn test_update_admin() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let msg = InstantiateMsg {
            params: RoundWhitelistFactoryParams {
                admin: Addr::unchecked("admin"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                whitelist_creation_fee: Coin::new(100, "uflix"),
                whitelist_code_id: 1,
                product_label: "product_label".to_string(),
            },
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Non admin can not update admin
        let info = mock_info("anyone", &[]);
        let res = update_admin(deps.as_mut(), env.clone(), info, "anyone".to_string());

        assert_eq!(res.err().unwrap(), ContractError::Unauthorized {});

        // Admin can update admin
        let info = mock_info("admin", &[]);
        let _res = update_admin(deps.as_mut(), env.clone(), info, "anyone".to_string()).unwrap();

        // Query params
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Params {}).unwrap();
        let params: ParamsResponse = from_json(res).unwrap();
        assert_eq!(params.params.admin, Addr::unchecked("anyone"));
    }

    #[test]
    fn test_update_fee_collector_address() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let msg = InstantiateMsg {
            params: RoundWhitelistFactoryParams {
                admin: Addr::unchecked("admin"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                whitelist_creation_fee: Coin::new(100, "uflix"),
                whitelist_code_id: 1,
                product_label: "product_label".to_string(),
            },
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Non admin can not update fee_collector_address
        let info = mock_info("anyone", &[]);
        let res =
            update_fee_collector_address(deps.as_mut(), env.clone(), info, "anyone".to_string());

        assert_eq!(res.err().unwrap(), ContractError::Unauthorized {});

        // Admin can update fee_collector_address
        let info = mock_info("admin", &[]);
        let _res =
            update_fee_collector_address(deps.as_mut(), env.clone(), info, "anyone".to_string())
                .unwrap();

        // Query params
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Params {}).unwrap();
        let params: ParamsResponse = from_json(res).unwrap();
        assert_eq!(
            params.params.fee_collector_address,
            Addr::unchecked("anyone")
        );
    }

    #[test]
    fn test_update_whitelist_creation_fee() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let msg = InstantiateMsg {
            params: RoundWhitelistFactoryParams {
                admin: Addr::unchecked("admin"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                whitelist_creation_fee: Coin::new(100, "uflix"),
                whitelist_code_id: 1,
                product_label: "product_label".to_string(),
            },
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Non admin can not update whitelist_creation_fee
        let info = mock_info("anyone", &[]);
        let res = update_whitelist_creation_fee(
            deps.as_mut(),
            env.clone(),
            info,
            Coin::new(200, "uflix"),
        );

        assert_eq!(res.err().unwrap(), ContractError::Unauthorized {});

        // Admin can update whitelist_creation_fee
        let info = mock_info("admin", &[]);
        let _res = update_whitelist_creation_fee(
            deps.as_mut(),
            env.clone(),
            info,
            Coin::new(200, "uflix"),
        )
        .unwrap();

        // Query params
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Params {}).unwrap();
        let params: ParamsResponse = from_json(res).unwrap();
        assert_eq!(
            params.params.whitelist_creation_fee,
            Coin::new(200, "uflix")
        );
    }

    #[test]
    fn test_update_whitelist_code_id() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let msg = InstantiateMsg {
            params: RoundWhitelistFactoryParams {
                admin: Addr::unchecked("admin"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                whitelist_creation_fee: Coin::new(100, "uflix"),
                whitelist_code_id: 1,
                product_label: "product_label".to_string(),
            },
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Non admin can not update whitelist_code_id
        let info = mock_info("anyone", &[]);
        let res = update_whitelist_code_id(deps.as_mut(), env.clone(), info, 2);

        assert_eq!(res.err().unwrap(), ContractError::Unauthorized {});

        // Admin can update whitelist_code_id
        let info = mock_info("admin", &[]);
        let _res = update_whitelist_code_id(deps.as_mut(), env.clone(), info, 2).unwrap();

        // Query params
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Params {}).unwrap();
        let params: ParamsResponse = from_json(res).unwrap();
        assert_eq!(params.params.whitelist_code_id, 2);
    }
}
