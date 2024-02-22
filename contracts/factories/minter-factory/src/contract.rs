use std::str::FromStr;

use crate::error::ContractError;
use crate::msg::{CreateMinterMsg, ExecuteMsg, InstantiateMsg, ParamsResponse, QueryMsg};
use crate::state::PARAMS;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128, WasmMsg,
};
use factory_types::check_payment;
use omniflix_std::types::omniflix::onft::v1beta1::OnftQuerier;
#[cfg(not(test))]
const CREATION_FEE: Uint128 = Uint128::new(0);
#[cfg(not(test))]
const CREATION_FEE_DENOM: &str = "";

#[cfg(test)]
const CREATION_FEE: Uint128 = Uint128::new(100_000_000);
#[cfg(test)]
const CREATION_FEE_DENOM: &str = "uflix";

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
    }
}

fn create_minter(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: CreateMinterMsg,
) -> Result<Response, ContractError> {
    let params = PARAMS.load(deps.storage)?;
    let nft_creation_fee: Coin = if CREATION_FEE == Uint128::new(0) {
        let onft_querier = OnftQuerier::new(&deps.querier);
        let params = onft_querier.params()?;
        let denom_creation_fee = params.params.unwrap().denom_creation_fee.unwrap();
        Coin {
            amount: Uint128::from_str(&denom_creation_fee.amount)?,
            denom: denom_creation_fee.denom,
        }
    } else {
        Coin {
            amount: CREATION_FEE,
            denom: CREATION_FEE_DENOM.to_string(),
        }
    };
    check_payment(
        &info.funds,
        &[nft_creation_fee.clone(), params.creation_fee.clone()],
    )?;
    let mut msgs = Vec::<CosmosMsg>::new();
    msgs.push(CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(msg.init.admin.to_string()),
        code_id: params.contract_id,
        msg: to_json_binary(&msg)?,
        funds: vec![nft_creation_fee],
        label: params.product_label,
    }));
    if params.creation_fee.amount > Uint128::new(0) {
        msgs.push(CosmosMsg::Bank(BankMsg::Send {
            amount: vec![params.creation_fee],
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
    params.contract_id = minter_code_id;
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
    use crate::msg::MinterInitExtention;

    use super::*;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Decimal, Empty, Timestamp,
    };
    use factory_types::CustomPaymentError;
    use minter_types::CollectionDetails;

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            params: factory_types::FactoryParams::<Empty> {
                admin: Addr::unchecked("admin"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                contract_id: 1,
                creation_fee: Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
                product_label: "omniflix-nft-minter".to_string(),
                init: Empty {},
            },
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query params
        let params = query_params(deps.as_ref()).unwrap();
        assert_eq!(
            params.params,
            factory_types::FactoryParams::<Empty> {
                admin: Addr::unchecked("admin"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                contract_id: 1,
                creation_fee: Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
                product_label: "omniflix-nft-minter".to_string(),
                init: Empty {},
            }
        );
    }

    #[test]
    fn test_execute_create_minter() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            params: factory_types::FactoryParams::<Empty> {
                admin: Addr::unchecked("admin"),
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                contract_id: 1,
                creation_fee: Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
                product_label: "omniflix-nft-minter".to_string(),
                init: Empty {},
            },
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        let collection_details = CollectionDetails {
            collection_name: "Collection Name".to_string(),
            description: Some("This is a collection of unique tokens.".to_string()),
            preview_uri: Some("https://example.com/preview".to_string()),
            schema: Some("https://example.com/schema".to_string()),
            symbol: "SYM".to_string(),
            id: "collection_id".to_string(),
            uri: Some("https://example.com/collection".to_string()),
            uri_hash: Some("".to_string()),
            data: Some("Additional data for the collection".to_string()),
            royalty_receivers: None,
        };
        let token_details = minter_types::TokenDetails {
            token_name: "Token Name".to_string(),
            description: Some("This is a unique token.".to_string()),
            base_token_uri: "https://example.com/token".to_string(),
            transferable: true,
            extensible: false,
            nsfw: false,
            royalty_ratio: Decimal::percent(10),
            preview_uri: Some("https://example.com/preview".to_string()),
            data: Some("Additional data for the token".to_string()),
        };
        // Send additional funds
        let msg = ExecuteMsg::CreateMinter {
            msg: CreateMinterMsg {
                collection_details: collection_details.clone(),
                init: MinterInitExtention {
                    admin: "admin".to_string(),
                    whitelist_address: None,
                    mint_price: Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                    start_time: Timestamp::from_seconds(0),
                    payment_collector: None,
                    per_address_limit: 3,
                    end_time: None,
                    num_tokens: 100,
                },
                token_details: token_details.clone(),
            },
        };

        let info = mock_info(
            "creator",
            &[
                Coin {
                    amount: Uint128::new(100_000_000),
                    denom: "uflix".to_string(),
                },
                Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
                Coin {
                    amount: Uint128::new(100),
                    denom: "additional".to_string(),
                },
            ],
        );
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(
            res,
            ContractError::PaymentError(CustomPaymentError::InsufficientFunds {
                expected: vec![
                    Coin {
                        amount: Uint128::new(100_000_000),
                        denom: "uflix".to_string(),
                    },
                    Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                ],
                actual: vec![
                    Coin {
                        amount: Uint128::new(100_000_000),
                        denom: "uflix".to_string(),
                    },
                    Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                    Coin {
                        amount: Uint128::new(100),
                        denom: "additional".to_string(),
                    },
                ],
            })
        );

        // Missing funds
        let msg = ExecuteMsg::CreateMinter {
            msg: CreateMinterMsg {
                collection_details: collection_details.clone(),
                token_details: token_details.clone(),
                init: MinterInitExtention {
                    admin: "admin".to_string(),
                    whitelist_address: None,
                    mint_price: Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                    start_time: Timestamp::from_seconds(0),
                    payment_collector: None,
                    per_address_limit: 3,
                    end_time: None,
                    num_tokens: 100,
                },
            },
        };

        let info = mock_info(
            "creator",
            &[Coin {
                amount: Uint128::new(100_000_000),
                denom: "uflix".to_string(),
            }],
        );
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(
            res,
            ContractError::PaymentError(CustomPaymentError::InsufficientFunds {
                expected: vec![
                    Coin {
                        amount: Uint128::new(100_000_000),
                        denom: "uflix".to_string(),
                    },
                    Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                ],
                actual: vec![Coin {
                    amount: Uint128::new(100_000_000),
                    denom: "uflix".to_string(),
                }],
            })
        );

        // Happy path
        let msg = ExecuteMsg::CreateMinter {
            msg: CreateMinterMsg {
                collection_details: collection_details.clone(),
                init: MinterInitExtention {
                    admin: "admin".to_string(),
                    whitelist_address: None,
                    mint_price: Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                    start_time: Timestamp::from_seconds(0),
                    payment_collector: None,
                    per_address_limit: 3,
                    end_time: None,
                    num_tokens: 100,
                },
                token_details: token_details.clone(),
            },
        };

        let info = mock_info(
            "creator",
            &[
                Coin {
                    amount: Uint128::new(100_000_000),
                    denom: "uflix".to_string(),
                },
                Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
            ],
        );
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(res.messages.len(), 2);
        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Wasm(WasmMsg::Instantiate {
                admin: Some("admin".to_string()),
                code_id: 1,
                msg: to_json_binary(&CreateMinterMsg {
                    collection_details: collection_details.clone(),
                    init: MinterInitExtention {
                        admin: "admin".to_string(),
                        whitelist_address: None,
                        mint_price: Coin {
                            amount: Uint128::new(100),
                            denom: "uusd".to_string(),
                        },
                        start_time: Timestamp::from_seconds(0),
                        payment_collector: None,
                        per_address_limit: 3,
                        end_time: None,
                        num_tokens: 100,
                    },
                    token_details: token_details.clone(),
                })
                .unwrap(),
                funds: vec![Coin {
                    amount: Uint128::new(100_000_000),
                    denom: "uflix".to_string(),
                }],
                label: "omniflix-nft-minter".to_string(),
            })
        );
        assert_eq!(
            res.messages[1].msg,
            CosmosMsg::Bank(BankMsg::Send {
                amount: vec![Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                }],
                to_address: "fee_collector_address".to_string(),
            })
        );
    }
}
