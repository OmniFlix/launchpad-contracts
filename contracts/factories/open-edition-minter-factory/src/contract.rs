use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, MultiMinterCreateMsg, MultiMinterFactoryExtension,
    OpenEditionMinterCreateMsg, ParamsResponse, QueryMsg,
};
use crate::state::PARAMS;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128, WasmMsg,
};
use factory_types::check_payment;
use minter_types::check_collection_creation_fee;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let _admin = deps
        .api
        .addr_validate(&msg.params.clone().admin.into_string())
        .unwrap_or(info.sender.clone());
    let _fee_collector_address = deps
        .api
        .addr_validate(&msg.params.fee_collector_address.clone().into_string())
        .unwrap_or(info.sender.clone());

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
        ExecuteMsg::UpdateMinterCodeId { minter_code_id } => {
            update_params_minter_code_id(deps, env, info, minter_code_id)
        }
        ExecuteMsg::UpdateMinterCreationFee {
            minter_creation_fee,
        } => update_params_minter_creation_fee(deps, env, info, minter_creation_fee),
        ExecuteMsg::UpdateMultiMinterCreationFee {
            multi_minter_creation_fee,
        } => update_params_multi_minter_creation_fee(deps, env, info, multi_minter_creation_fee),
        ExecuteMsg::UpdateMultiMinterCodeId {
            multi_minter_code_id,
        } => update_params_multi_minter_code_id(deps, env, info, multi_minter_code_id),
    }
}

fn create_oem(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: OpenEditionMinterCreateMsg,
) -> Result<Response, ContractError> {
    let params = PARAMS.load(deps.storage)?;
    let collection_creation_fee: Coin = check_collection_creation_fee(deps.as_ref().querier)?;

    check_payment(
        &info.funds,
        &[collection_creation_fee.clone(), params.creation_fee.clone()],
    )?;
    let mut msgs = Vec::<CosmosMsg>::new();

    msgs.push(CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(msg.init.admin.to_string()),
        code_id: params.code_id,
        msg: to_json_binary(&msg)?,
        funds: vec![collection_creation_fee.clone()],
        label: params.product_label,
    }));
    if params.creation_fee.amount > Uint128::new(0) {
        msgs.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: params.fee_collector_address.to_string(),
            amount: vec![params.creation_fee.clone()],
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
    let params = PARAMS.load(deps.storage)?;
    let collection_creation_fee: Coin = check_collection_creation_fee(deps.as_ref().querier)?;

    check_payment(
        &info.funds,
        &[
            collection_creation_fee.clone(),
            params.init.multi_minter_creation_fee.clone(),
        ],
    )?;
    let mut msgs = Vec::<CosmosMsg>::new();

    msgs.push(CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(msg.init.admin.to_string()),
        code_id: params.init.multi_minter_code_id,
        msg: to_json_binary(&msg)?,
        funds: vec![collection_creation_fee.clone()],
        label: params.product_label,
    }));
    if params.init.multi_minter_creation_fee.amount > Uint128::new(0) {
        msgs.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: params.fee_collector_address.to_string(),
            amount: vec![params.init.multi_minter_creation_fee.clone()],
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
    params.code_id = minter_code_id;
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
    params.creation_fee = minter_creation_fee.clone();

    PARAMS.save(deps.storage, &params)?;
    Ok(Response::default()
        .add_attribute("action", "update_minter_creation_fee")
        .add_attribute("new_minter_creation_fee", minter_creation_fee.to_string()))
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
    params.init.multi_minter_creation_fee = multi_minter_creation_fee.clone();

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
    params.init.multi_minter_code_id = multi_minter_code_id;
    PARAMS.save(deps.storage, &params)?;
    Ok(Response::default()
        .add_attribute("action", "update_multi_minter_code_id")
        .add_attribute("new_multi_minter_code_id", multi_minter_code_id.to_string()))
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

#[cfg(test)]
mod tests {

    use crate::msg::MultiMinterFactoryExtension;

    use super::*;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr,
    };

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            params: factory_types::FactoryParams::<MultiMinterFactoryExtension> {
                admin: Addr::unchecked("creator"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                code_id: 1,
                creation_fee: Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
                product_label: "omniflix-open-edition-minter".to_string(),
                init: MultiMinterFactoryExtension {
                    multi_minter_code_id: 1,
                    multi_minter_creation_fee: Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                },
            },
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query params
        let params = query_params(deps.as_ref()).unwrap();
        assert_eq!(
            params.params,
            factory_types::FactoryParams::<MultiMinterFactoryExtension> {
                admin: Addr::unchecked("creator"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                code_id: 1,
                creation_fee: Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
                product_label: "omniflix-open-edition-minter".to_string(),
                init: MultiMinterFactoryExtension {
                    multi_minter_code_id: 1,
                    multi_minter_creation_fee: Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                },
            }
        );
    }
}
