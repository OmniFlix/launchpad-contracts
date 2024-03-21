use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, MultiMinterCreateMsg, OpenEditionMinterCreateMsg, ParamsResponse,
    QueryMsg,
};
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
        ExecuteMsg::CreateOpenEditionMinter { msg } => create_oem(deps, env, info, msg),
        ExecuteMsg::CreateMultiMintOpenEditionMinter { msg } => {
            create_multi_mint_oem(deps, env, info, msg)
        }
        ExecuteMsg::UpdateAdmin { admin } => update_params_admin(deps, env, info, admin),
        ExecuteMsg::UpdateFeeCollectorAddress {
            fee_collector_address,
        } => update_params_fee_collector_address(deps, env, info, fee_collector_address),
        ExecuteMsg::UpdateOpenEditionMinterCodeId {
            open_edition_minter_code_id,
        } => {
            update_params_open_edition_minter_code_id(deps, env, info, open_edition_minter_code_id)
        }
        ExecuteMsg::UpdateOpenEditionMinterCreationFee {
            open_edition_minter_creation_fee,
        } => update_params_open_edition_minter_creation_fee(
            deps,
            env,
            info,
            open_edition_minter_creation_fee,
        ),
        ExecuteMsg::UpdateMultiMinterCreationFee {
            multi_minter_creation_fee,
        } => update_params_multi_minter_creation_fee(deps, env, info, multi_minter_creation_fee),
        ExecuteMsg::UpdateMultiMinterCodeId {
            multi_minter_code_id,
        } => update_params_multi_minter_code_id(deps, env, info, multi_minter_code_id),
        ExecuteMsg::Pause {} => execute_pause(deps, env, info),
        ExecuteMsg::Unpause {} => execute_unpause(deps, env, info),
        ExecuteMsg::SetPausers { pausers } => set_pausers(deps, env, info, pausers),
    }
}

fn create_oem(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: OpenEditionMinterCreateMsg,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.as_ref().storage)?;
    let params = PARAMS.load(deps.storage)?;
    let collection_creation_fee: Coin = check_collection_creation_fee(deps.as_ref().querier)?;
    check_payment(
        &info.funds,
        &[
            collection_creation_fee.clone(),
            params.open_edition_minter_creation_fee.clone(),
        ],
    )?;
    let mut msgs = Vec::<CosmosMsg>::new();

    msgs.push(CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(msg.init.admin.to_string()),
        code_id: params.open_edition_minter_code_id,
        msg: to_json_binary(&msg)?,
        funds: vec![collection_creation_fee.clone()],
        label: params.oem_product_label,
    }));
    if params.open_edition_minter_creation_fee.amount > Uint128::new(0) {
        msgs.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: params.fee_collector_address.to_string(),
            amount: vec![params.open_edition_minter_creation_fee.clone()],
        }));
    }
    let res = Response::new()
        .add_messages(msgs)
        .add_attribute("action", "create_minter");
    Ok(res)
}

fn create_multi_mint_oem(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MultiMinterCreateMsg,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.as_ref().storage)?;
    let params = PARAMS.load(deps.storage)?;
    if params.multi_minter_params.is_none() {
        return Err(ContractError::MultiMinterNotEnabled {});
    }
    let multi_minter_params = params.multi_minter_params.unwrap();
    let collection_creation_fee: Coin = check_collection_creation_fee(deps.as_ref().querier)?;

    check_payment(
        &info.funds,
        &[
            collection_creation_fee.clone(),
            multi_minter_params.multi_minter_creation_fee.clone(),
        ],
    )?;
    let mut msgs = Vec::<CosmosMsg>::new();

    msgs.push(CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(msg.init.admin.to_string()),
        code_id: multi_minter_params.multi_minter_code_id,
        msg: to_json_binary(&msg)?,
        funds: vec![collection_creation_fee.clone()],
        label: multi_minter_params.multi_minter_product_label,
    }));
    if multi_minter_params.multi_minter_creation_fee.amount > Uint128::new(0) {
        msgs.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: params.fee_collector_address.to_string(),
            amount: vec![multi_minter_params.multi_minter_creation_fee.clone()],
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

fn update_params_open_edition_minter_code_id(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    open_edition_minter_code_id: u64,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;
    if params.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    params.open_edition_minter_code_id = open_edition_minter_code_id;
    PARAMS.save(deps.storage, &params)?;
    Ok(Response::default()
        .add_attribute("action", "update_minter_code_id")
        .add_attribute(
            "new_minter_code_id",
            open_edition_minter_code_id.to_string(),
        ))
}

fn update_params_open_edition_minter_creation_fee(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    open_edition_minter_creation_fee: Coin,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;
    if params.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    params.open_edition_minter_creation_fee = open_edition_minter_creation_fee.clone();

    PARAMS.save(deps.storage, &params)?;
    Ok(Response::default()
        .add_attribute("action", "update_minter_creation_fee")
        .add_attribute(
            "new_minter_creation_fee",
            open_edition_minter_creation_fee.to_string(),
        ))
}

fn update_params_multi_minter_creation_fee(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    multi_minter_creation_fee: Coin,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;
    if params.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    if params.multi_minter_params.is_none() {
        return Err(ContractError::MultiMinterNotEnabled {});
    }
    params
        .multi_minter_params
        .as_mut()
        .unwrap()
        .multi_minter_creation_fee = multi_minter_creation_fee.clone();

    PARAMS.save(deps.storage, &params)?;
    Ok(Response::default()
        .add_attribute("action", "update_multi_minter_creation_fee")
        .add_attribute(
            "new_multi_minter_creation_fee",
            multi_minter_creation_fee.to_string(),
        ))
}

fn update_params_multi_minter_code_id(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    multi_minter_code_id: u64,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;
    if params.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    if params.multi_minter_params.is_none() {
        return Err(ContractError::MultiMinterNotEnabled {});
    }
    params
        .multi_minter_params
        .as_mut()
        .unwrap()
        .multi_minter_code_id = multi_minter_code_id;

    PARAMS.save(deps.storage, &params)?;
    Ok(Response::default()
        .add_attribute("action", "update_multi_minter_code_id")
        .add_attribute("new_multi_minter_code_id", multi_minter_code_id.to_string()))
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
mod open_edition_minter_factory_test {
    use crate::msg::{MultiMinterParams, OpenEditionMinterFactoryParams};

    use super::*;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr,
    };
    use pauser::PauseError;

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            params: OpenEditionMinterFactoryParams {
                admin: Addr::unchecked("creator"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                multi_minter_params: Some(MultiMinterParams {
                    multi_minter_code_id: 1,
                    multi_minter_creation_fee: Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                    multi_minter_product_label: "omniflix-multi-minter".to_string(),
                }),
                open_edition_minter_code_id: 1,
                open_edition_minter_creation_fee: Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
                oem_product_label: "omniflix-open-edition-minter".to_string(),
            },
        };

        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query params
        let params = query_params(deps.as_ref()).unwrap();
        assert_eq!(
            params.params,
            OpenEditionMinterFactoryParams {
                admin: Addr::unchecked("creator"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                multi_minter_params: Some(MultiMinterParams {
                    multi_minter_code_id: 1,
                    multi_minter_creation_fee: Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                    multi_minter_product_label: "omniflix-multi-minter".to_string(),
                }),
                open_edition_minter_code_id: 1,
                open_edition_minter_creation_fee: Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
                oem_product_label: "omniflix-open-edition-minter".to_string(),
            }
        );
    }
    #[test]
    fn test_update_admin() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            params: OpenEditionMinterFactoryParams {
                admin: Addr::unchecked("admin"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                multi_minter_params: Some(MultiMinterParams {
                    multi_minter_code_id: 1,
                    multi_minter_creation_fee: Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                    multi_minter_product_label: "omniflix-multi-minter".to_string(),
                }),
                open_edition_minter_code_id: 1,
                open_edition_minter_creation_fee: Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
                oem_product_label: "omniflix-open-edition-minter".to_string(),
            },
        };

        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Non admin can not update admin
        let info = mock_info("non_admin", &[]);
        let msg = ExecuteMsg::UpdateAdmin {
            admin: "new_admin".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert_eq!(res.unwrap_err(), ContractError::Unauthorized {});

        // admin can update admin
        let info = mock_info("admin", &[]);
        let msg = ExecuteMsg::UpdateAdmin {
            admin: "new_admin".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query params
        let params = query_params(deps.as_ref()).unwrap();
        assert_eq!(params.params.admin, Addr::unchecked("new_admin"));
    }

    #[test]
    fn test_update_fee_collector_address() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            params: OpenEditionMinterFactoryParams {
                admin: Addr::unchecked("admin"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                multi_minter_params: Some(MultiMinterParams {
                    multi_minter_code_id: 1,
                    multi_minter_creation_fee: Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                    multi_minter_product_label: "omniflix-multi-minter".to_string(),
                }),
                open_edition_minter_code_id: 1,
                open_edition_minter_creation_fee: Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
                oem_product_label: "omniflix-open-edition-minter".to_string(),
            },
        };

        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Non admin can not update fee_collector_address
        let info = mock_info("non_admin", &[]);
        let msg = ExecuteMsg::UpdateFeeCollectorAddress {
            fee_collector_address: "new_fee_collector_address".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert_eq!(res.unwrap_err(), ContractError::Unauthorized {});

        // admin can update fee_collector_address
        let info = mock_info("admin", &[]);
        let msg = ExecuteMsg::UpdateFeeCollectorAddress {
            fee_collector_address: "new_fee_collector_address".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query params
        let params = query_params(deps.as_ref()).unwrap();
        assert_eq!(
            params.params.fee_collector_address,
            Addr::unchecked("new_fee_collector_address")
        );
    }

    #[test]
    fn test_update_open_edition_minter_code_id() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            params: OpenEditionMinterFactoryParams {
                admin: Addr::unchecked("admin"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                multi_minter_params: Some(MultiMinterParams {
                    multi_minter_code_id: 1,
                    multi_minter_creation_fee: Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                    multi_minter_product_label: "omniflix-multi-minter".to_string(),
                }),
                open_edition_minter_code_id: 1,
                open_edition_minter_creation_fee: Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
                oem_product_label: "omniflix-open-edition-minter".to_string(),
            },
        };

        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Non admin can not update open_edition_minter_code_id
        let info = mock_info("non_admin", &[]);
        let msg = ExecuteMsg::UpdateOpenEditionMinterCodeId {
            open_edition_minter_code_id: 2,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert_eq!(res.unwrap_err(), ContractError::Unauthorized {});

        // admin can update open_edition_minter_code_id
        let info = mock_info("admin", &[]);
        let msg = ExecuteMsg::UpdateOpenEditionMinterCodeId {
            open_edition_minter_code_id: 2,
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query params
        let params = query_params(deps.as_ref()).unwrap();
        assert_eq!(params.params.open_edition_minter_code_id, 2);
    }

    #[test]
    fn test_update_open_edition_minter_creation_fee() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            params: OpenEditionMinterFactoryParams {
                admin: Addr::unchecked("admin"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                multi_minter_params: Some(MultiMinterParams {
                    multi_minter_code_id: 1,
                    multi_minter_creation_fee: Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                    multi_minter_product_label: "omniflix-multi-minter".to_string(),
                }),
                open_edition_minter_code_id: 1,
                open_edition_minter_creation_fee: Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
                oem_product_label: "omniflix-open-edition-minter".to_string(),
            },
        };

        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Non admin can not update open_edition_minter_creation_fee
        let info = mock_info("non_admin", &[]);
        let msg = ExecuteMsg::UpdateOpenEditionMinterCreationFee {
            open_edition_minter_creation_fee: Coin {
                amount: Uint128::new(200),
                denom: "uusd".to_string(),
            },
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert_eq!(res.unwrap_err(), ContractError::Unauthorized {});

        // admin can update open_edition_minter_creation_fee
        let info = mock_info("admin", &[]);
        let msg = ExecuteMsg::UpdateOpenEditionMinterCreationFee {
            open_edition_minter_creation_fee: Coin {
                amount: Uint128::new(200),
                denom: "uusd".to_string(),
            },
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query params
        let params = query_params(deps.as_ref()).unwrap();
        assert_eq!(
            params.params.open_edition_minter_creation_fee,
            Coin {
                amount: Uint128::new(200),
                denom: "uusd".to_string()
            }
        );
    }

    #[test]
    fn test_update_multi_minter_creation_fee() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            params: OpenEditionMinterFactoryParams {
                admin: Addr::unchecked("admin"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                multi_minter_params: Some(MultiMinterParams {
                    multi_minter_code_id: 1,
                    multi_minter_creation_fee: Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                    multi_minter_product_label: "omniflix-multi-minter".to_string(),
                }),
                open_edition_minter_code_id: 1,
                open_edition_minter_creation_fee: Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
                oem_product_label: "omniflix-open-edition-minter".to_string(),
            },
        };

        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Non admin can not update multi_minter_creation_fee
        let info = mock_info("non_admin", &[]);
        let msg = ExecuteMsg::UpdateMultiMinterCreationFee {
            multi_minter_creation_fee: Coin {
                amount: Uint128::new(200),
                denom: "uusd".to_string(),
            },
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert_eq!(res.unwrap_err(), ContractError::Unauthorized {});

        // admin can update multi_minter_creation_fee
        let info = mock_info("admin", &[]);
        let msg = ExecuteMsg::UpdateMultiMinterCreationFee {
            multi_minter_creation_fee: Coin {
                amount: Uint128::new(200),
                denom: "uusd".to_string(),
            },
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query params
        let params = query_params(deps.as_ref()).unwrap();
        assert_eq!(
            params
                .params
                .multi_minter_params
                .unwrap()
                .multi_minter_creation_fee,
            Coin {
                amount: Uint128::new(200),
                denom: "uusd".to_string(),
            }
        );
    }

    #[test]
    fn test_update_multi_minter_code_id() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            params: OpenEditionMinterFactoryParams {
                admin: Addr::unchecked("admin"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                multi_minter_params: Some(MultiMinterParams {
                    multi_minter_code_id: 1,
                    multi_minter_creation_fee: Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                    multi_minter_product_label: "omniflix-multi-minter".to_string(),
                }),
                open_edition_minter_code_id: 1,
                open_edition_minter_creation_fee: Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
                oem_product_label: "omniflix-open-edition-minter".to_string(),
            },
        };

        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Non admin can not update multi_minter_code_id
        let info = mock_info("non_admin", &[]);
        let msg = ExecuteMsg::UpdateMultiMinterCodeId {
            multi_minter_code_id: 2,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert_eq!(res.unwrap_err(), ContractError::Unauthorized {});

        // admin can update multi_minter_code_id
        let info = mock_info("admin", &[]);
        let msg = ExecuteMsg::UpdateMultiMinterCodeId {
            multi_minter_code_id: 2,
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query params
        let params = query_params(deps.as_ref()).unwrap();
        assert_eq!(
            params
                .params
                .multi_minter_params
                .unwrap()
                .multi_minter_code_id,
            2
        );
    }

    #[test]
    fn test_execute_pause_unpause() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            params: OpenEditionMinterFactoryParams {
                admin: Addr::unchecked("admin"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                multi_minter_params: Some(MultiMinterParams {
                    multi_minter_code_id: 1,
                    multi_minter_creation_fee: Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                    multi_minter_product_label: "omniflix-multi-minter".to_string(),
                }),
                open_edition_minter_code_id: 1,
                open_edition_minter_creation_fee: Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
                oem_product_label: "omniflix-open-edition-minter".to_string(),
            },
        };

        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Non pauser can not pause
        let info = mock_info("non_pauser", &[]);
        let msg = ExecuteMsg::Pause {};
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert_eq!(
            res.unwrap_err(),
            ContractError::Pause(PauseError::Unauthorized {
                sender: Addr::unchecked("non_pauser")
            })
        );

        // pauser can pause
        let info = mock_info("admin", &[]);
        let msg = ExecuteMsg::Pause {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query is_paused
        let is_paused = query_is_paused(deps.as_ref(), mock_env()).unwrap();
        assert_eq!(is_paused, true);

        // Non pauser can not unpause
        let info = mock_info("non_pauser", &[]);
        let msg = ExecuteMsg::Unpause {};
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert_eq!(
            res.unwrap_err(),
            ContractError::Pause(PauseError::Unauthorized {
                sender: Addr::unchecked("non_pauser")
            })
        );

        // pauser can unpause
        let info = mock_info("admin", &[]);
        let msg = ExecuteMsg::Unpause {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query is_paused
        let is_paused = query_is_paused(deps.as_ref(), mock_env()).unwrap();
        assert_eq!(is_paused, false);
    }

    #[test]
    fn test_set_pausers() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            params: OpenEditionMinterFactoryParams {
                admin: Addr::unchecked("admin"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                multi_minter_params: Some(MultiMinterParams {
                    multi_minter_code_id: 1,
                    multi_minter_creation_fee: Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                    multi_minter_product_label: "omniflix-multi-minter".to_string(),
                }),
                open_edition_minter_code_id: 1,
                open_edition_minter_creation_fee: Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
                oem_product_label: "omniflix-open-edition-minter".to_string(),
            },
        };

        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Non pauser can not set pausers
        let info = mock_info("non_pauser", &[]);
        let msg = ExecuteMsg::SetPausers {
            pausers: vec!["pauser1".to_string(), "pauser2".to_string()],
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert_eq!(
            res.unwrap_err(),
            ContractError::Pause(PauseError::Unauthorized {
                sender: Addr::unchecked("non_pauser")
            })
        );

        // pauser can set pausers
        let info = mock_info("admin", &[]);
        let msg = ExecuteMsg::SetPausers {
            pausers: vec!["pauser1".to_string(), "pauser2".to_string()],
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query pausers
        let pausers = query_pausers(deps.as_ref(), mock_env()).unwrap();
        assert_eq!(
            pausers,
            vec![Addr::unchecked("pauser1"), Addr::unchecked("pauser2"),]
        );
    }
}
