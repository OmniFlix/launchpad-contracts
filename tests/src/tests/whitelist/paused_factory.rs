#![cfg(test)]
use crate::helpers::mock_messages::factory_mock_messages::return_round_whitelist_factory_inst_message;
use crate::helpers::mock_messages::whitelist_mock_messages::return_rounds;
use crate::helpers::setup::{setup, SetupResponse};
use cosmwasm_std::{coin, to_json_binary, QueryRequest, WasmQuery};

use cw_multi_test::Executor;
use omniflix_round_whitelist_factory::error::ContractError as RoundWhitelistFactoryContractError;
use omniflix_round_whitelist_factory::msg::{
    ExecuteMsg as RoundWhitelistFactoryExecuteMsgs, QueryMsg as RoundWhitelistFactoryQueryMsgs,
};

#[test]
fn paused_factory() {
    let res: SetupResponse = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let round_whitelist_factory_code_id = res.round_whitelist_factory_code_id;
    let round_whitelist_code_id = res.round_whitelist_code_id;
    let mut app = res.app;

    let round_whitelist_factory_inst_msg =
        return_round_whitelist_factory_inst_message(round_whitelist_code_id);

    let round_whitelist_factory_addr = app
        .instantiate_contract(
            round_whitelist_factory_code_id,
            admin.clone(),
            &round_whitelist_factory_inst_msg,
            &[],
            "round_whitelist_factory",
            None,
        )
        .unwrap();

    let is_paused: bool = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: round_whitelist_factory_addr.clone().into_string(),
            msg: to_json_binary(&RoundWhitelistFactoryQueryMsgs::IsPaused {}).unwrap(),
        }))
        .unwrap();
    assert_eq!(is_paused, false);

    // Create a whitelist
    let rounds = return_rounds();
    let _res = app
        .execute_contract(
            creator.clone(),
            round_whitelist_factory_addr.clone(),
            &omniflix_round_whitelist_factory::msg::ExecuteMsg::CreateWhitelist {
                msg: whitelist_types::InstantiateMsg {
                    admin: admin.to_string(),
                    rounds: rounds.clone(),
                },
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap();
    // Pause the factory
    let _res = app
        .execute_contract(
            admin.clone(),
            round_whitelist_factory_addr.clone(),
            &RoundWhitelistFactoryExecuteMsgs::Pause {},
            &[],
        )
        .unwrap();
    let is_paused: bool = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: round_whitelist_factory_addr.clone().into_string(),
            msg: to_json_binary(&RoundWhitelistFactoryQueryMsgs::IsPaused {}).unwrap(),
        }))
        .unwrap();
    assert_eq!(is_paused, true);

    // Try creating a whitelist while factory is paused
    let error = app
        .execute_contract(
            creator.clone(),
            round_whitelist_factory_addr.clone(),
            &omniflix_round_whitelist_factory::msg::ExecuteMsg::CreateWhitelist {
                msg: whitelist_types::InstantiateMsg {
                    admin: admin.to_string(),
                    rounds: rounds.clone(),
                },
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res
        .downcast_ref::<RoundWhitelistFactoryContractError>()
        .unwrap();
    assert_eq!(
        error,
        &RoundWhitelistFactoryContractError::Pause(pauser::PauseError::Paused {})
    );
}
