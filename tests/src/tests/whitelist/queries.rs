#![cfg(test)]
use crate::helpers::mock_messages::factory_mock_messages::return_round_whitelist_factory_inst_message;
use crate::helpers::mock_messages::whitelist_mock_messages::return_round_configs;
use crate::helpers::setup::{setup, SetupResponse};
use crate::helpers::utils::get_contract_address_from_res;
use cosmwasm_std::{
    coin, to_json_binary, Addr, BlockInfo, Coin, QueryRequest, StdError, Timestamp, WasmQuery,
};

use cw_multi_test::Executor;
use whitelist_types::{CreateWhitelistMsg, Round, RoundWhitelistQueryMsgs};

#[test]
fn whitelist_queries() {
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
    let res = app
        .execute_contract(
            creator.clone(),
            round_whitelist_factory_addr.clone(),
            &omniflix_round_whitelist_factory::msg::ExecuteMsg::CreateWhitelist {
                msg: CreateWhitelistMsg {
                    admin: admin.to_string(),
                    rounds: rounds.clone(),
                },
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap();
    let round_whitelist_address = get_contract_address_from_res(res);

    // Query config
    let config_data: String = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: round_whitelist_address.clone(),
            msg: to_json_binary(&RoundWhitelistQueryMsgs::Admin {}).unwrap(),
        }))
        .unwrap();
    assert_eq!(config_data, admin.to_string());
    // Query rounds
    let rounds_data: Vec<(u32, Round)> = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: round_whitelist_address.clone(),
            msg: to_json_binary(&RoundWhitelistQueryMsgs::Rounds {}).unwrap(),
        }))
        .unwrap();
    assert_eq!(rounds_data.len(), 2);
    assert_eq!(rounds_data[0].1.start_time, Timestamp::from_nanos(2000));
    assert_eq!(rounds_data[0].1.end_time, Timestamp::from_nanos(3000));

    // Query round by id
    let round_data: Round = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: round_whitelist_address.clone(),
            msg: to_json_binary(&RoundWhitelistQueryMsgs::Round { round_index: 1 }).unwrap(),
        }))
        .unwrap();
    assert_eq!(round_data.start_time, Timestamp::from_nanos(2000));
    assert_eq!(round_data.end_time, Timestamp::from_nanos(3000));

    // Query round by id
    let round_data: Round = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: round_whitelist_address.clone(),
            msg: to_json_binary(&RoundWhitelistQueryMsgs::Round { round_index: 2 }).unwrap(),
        }))
        .unwrap();
    assert_eq!(round_data.start_time, Timestamp::from_nanos(4000));
    assert_eq!(round_data.end_time, Timestamp::from_nanos(5000));

    // Query active round should return error
    let res: Result<Round, StdError> = app.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: round_whitelist_address.clone(),
        msg: to_json_binary(&RoundWhitelistQueryMsgs::ActiveRound {}).unwrap(),
    }));
    assert!(res.is_err());

    // Change time to 2000
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_nanos(2000),
        chain_id: "test_1".to_string(),
    });

    // Query active round
    let round_data: (u32, Round) = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: round_whitelist_address.clone(),
            msg: to_json_binary(&RoundWhitelistQueryMsgs::ActiveRound {}).unwrap(),
        }))
        .unwrap();
    assert_eq!(round_data.1.start_time, Timestamp::from_nanos(2000));
    assert_eq!(round_data.1.end_time, Timestamp::from_nanos(3000));

    // Query price should be first round price
    let price: Coin = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: round_whitelist_address.clone(),
            msg: to_json_binary(&RoundWhitelistQueryMsgs::Price {}).unwrap(),
        }))
        .unwrap();
    assert_eq!(price, rounds[0].round.mint_price);

    // Query is_member
    // Creator is not a member of first round
    let is_member: bool = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: round_whitelist_address.clone(),
            msg: to_json_binary(&RoundWhitelistQueryMsgs::IsMember {
                address: creator.to_string(),
            })
            .unwrap(),
        }))
        .unwrap();
    assert!(!is_member);

    // Query is_member for collector should return true
    let is_member: bool = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: round_whitelist_address.clone(),
            msg: to_json_binary(&RoundWhitelistQueryMsgs::IsMember {
                address: Addr::unchecked("collector".to_string()).to_string(),
            })
            .unwrap(),
        }))
        .unwrap();
    assert!(is_member);

    // Query members
}

#[test]
fn test_paginated_members_query() {
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
    let mut rounds = return_round_configs();
    let mut additional_members = vec![];
    for i in 0..1000 {
        additional_members.push(format!("member_{}", i));
    }

    rounds[0].members = additional_members.clone();

    let res = app
        .execute_contract(
            creator.clone(),
            round_whitelist_factory_addr.clone(),
            &omniflix_round_whitelist_factory::msg::ExecuteMsg::CreateWhitelist {
                msg: CreateWhitelistMsg {
                    admin: admin.to_string(),
                    rounds: rounds.clone(),
                },
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap();

    let round_whitelist_address = get_contract_address_from_res(res);

    // Query round 1 members
    // Create a loop to paginate and get all members
    let mut members: Vec<String> = vec![];
    let mut start_after: Option<String> = None;
    let limit = 100;
    loop {
        let members_data: Vec<String> = app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: round_whitelist_address.clone(),
                msg: to_json_binary(&RoundWhitelistQueryMsgs::Members {
                    round_index: 1,
                    start_after: start_after.clone(),
                    limit: Some(limit),
                })
                .unwrap(),
            }))
            .unwrap();
        if members_data.is_empty() {
            break;
        }
        start_after = Some(members_data.last().unwrap().clone());
        members.extend(members_data);
    }

    // Sort members by extracting the numeric part
    members.sort_unstable_by_key(|member| {
        member
            .trim_start_matches("member_")
            .parse::<usize>()
            .unwrap()
    });

    assert_eq!(members.len(), 1000);
    assert_eq!(members, additional_members);
}
