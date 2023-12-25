#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    to_json_binary, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdResult, Timestamp, Uint128, WasmMsg,
};
use cw_utils::{maybe_addr, must_pay, nonpayable};
