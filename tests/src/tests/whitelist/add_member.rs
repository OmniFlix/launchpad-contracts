#![cfg(test)]
use crate::helpers::mock_messages::factory_mock_messages::return_round_whitelist_factory_inst_message;
use crate::helpers::mock_messages::whitelist_mock_messages::return_round_configs;
use crate::helpers::setup::{setup, SetupResponse};
use crate::helpers::utils::get_contract_address_from_res;
use cosmwasm_std::{coin, Addr};

use cw_multi_test::Executor;
use omniflix_round_whitelist::error::ContractError as RoundWhitelistContractError;
use omniflix_round_whitelist::msg::ExecuteMsg;
use whitelist_types::{CreateWhitelistMsg, RoundWhitelistQueryMsgs};

#[test]
fn add_member() {
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
    let rounds = return_round_configs();
    // Create a whitelist
    let res = app
        .execute_contract(
            creator.clone(),
            round_whitelist_factory_addr.clone(),
            &omniflix_round_whitelist_factory::msg::ExecuteMsg::CreateWhitelist {
                msg: CreateWhitelistMsg {
                    admin: creator.to_string(),
                    rounds: rounds.clone(),
                },
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap();
    let round_whitelist_address = get_contract_address_from_res(res);

    // Non creator can not add member
    let res = app
        .execute_contract(
            admin.clone(),
            Addr::unchecked(round_whitelist_address.clone()),
            &ExecuteMsg::AddMembers {
                members: ["address".to_string()].to_vec(),
                round_index: 1,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<RoundWhitelistContractError>().unwrap();
    assert_eq!(error, &RoundWhitelistContractError::Unauthorized {});

    // Send wrong round index
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(round_whitelist_address.clone()),
            &ExecuteMsg::AddMembers {
                members: ["address".to_string()].to_vec(),
                round_index: 100,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<RoundWhitelistContractError>().unwrap();
    assert_eq!(error, &RoundWhitelistContractError::RoundNotFound {});

    // Curently we have 2 rounds.
    // First round has collector and second has creator whitelisted

    // Try adding collector to first round
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(round_whitelist_address.clone()),
            &ExecuteMsg::AddMembers {
                members: ["collector".to_string()].to_vec(),
                round_index: 1,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap();
    // Query members
    let members: Vec<String> = app
        .wrap()
        .query_wasm_smart(
            round_whitelist_address.clone(),
            &RoundWhitelistQueryMsgs::Members {
                round_index: 1,
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(members, vec![("collector".to_string())]);

    // Try adding 500 same address to first round
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(round_whitelist_address.clone()),
            &ExecuteMsg::AddMembers {
                members: vec!["collector".to_string(); 500],
                round_index: 1,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap();
    // Query members
    let members: Vec<String> = app
        .wrap()
        .query_wasm_smart(
            round_whitelist_address.clone(),
            &RoundWhitelistQueryMsgs::Members {
                round_index: 1,
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(members, vec![("collector".to_string())]);

    // Try adding 500 different addresses to first round
    let mut addresses: Vec<String> = Vec::new();
    for i in 0..500 {
        let address = format!("collector{}", i);
        addresses.push(address.clone());
    }
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(round_whitelist_address.clone()),
            &ExecuteMsg::AddMembers {
                members: addresses.clone(),
                round_index: 1,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap();
    // Query members
    let members: Vec<String> = app
        .wrap()
        .query_wasm_smart(
            round_whitelist_address.clone(),
            &RoundWhitelistQueryMsgs::Members {
                round_index: 1,
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
    // Default limit is 100
    assert_eq!(members.len(), 100);
    // Query members with limit
    let members: Vec<String> = app
        .wrap()
        .query_wasm_smart(
            round_whitelist_address.clone(),
            &RoundWhitelistQueryMsgs::Members {
                round_index: 1,
                start_after: None,
                limit: Some(49),
            },
        )
        .unwrap();
    assert_eq!(members.len(), 49);

    // Query members with start_after
    let members: Vec<String> = app
        .wrap()
        .query_wasm_smart(
            round_whitelist_address.clone(),
            &RoundWhitelistQueryMsgs::Members {
                round_index: 1,
                start_after: Some("collector150".to_string()),
                limit: Some(100),
            },
        )
        .unwrap();
    assert_eq!(members.len(), 100);
    assert_eq!(members[0], "collector151".to_string());

    // Query members with start_after and limit
    let members: Vec<String> = app
        .wrap()
        .query_wasm_smart(
            round_whitelist_address.clone(),
            &RoundWhitelistQueryMsgs::Members {
                round_index: 1,
                start_after: Some("collector150".to_string()),
                limit: Some(49),
            },
        )
        .unwrap();

    assert_eq!(members.len(), 49);
    let one_before_last_member = members[47].clone();
    let last_member = members[48].clone();

    // Paginate
    let members: Vec<String> = app
        .wrap()
        .query_wasm_smart(
            round_whitelist_address.clone(),
            &RoundWhitelistQueryMsgs::Members {
                round_index: 1,
                start_after: Some(one_before_last_member.clone()),
                limit: Some(49),
            },
        )
        .unwrap();
    assert_eq!(members.len(), 49);
    assert_eq!(members[0], last_member);

    // Member limit for an execution is 5000
    // Try adding more than 5000 members
    let mut addresses: Vec<String> = Vec::new();
    for i in 0..4999 + 2 {
        let address = format!("collector{}", i);
        addresses.push(address.clone());
    }
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(round_whitelist_address.clone()),
            &ExecuteMsg::AddMembers {
                members: addresses.clone(),
                round_index: 1,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<RoundWhitelistContractError>().unwrap();
    assert_eq!(
        error,
        &RoundWhitelistContractError::WhitelistMemberLimitExceeded {}
    );
}
