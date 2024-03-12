#![cfg(test)]
use crate::helpers::mock_messages::factory_mock_messages::return_round_whitelist_factory_inst_message;
use crate::helpers::mock_messages::whitelist_mock_messages::return_rounds;
use crate::helpers::setup::{setup, SetupResponse};
use crate::helpers::utils::get_contract_address_from_res;
use cosmwasm_std::{coin, to_json_binary, Addr, BlockInfo, QueryRequest, Timestamp, WasmQuery};

use cw_multi_test::Executor;
use omniflix_round_whitelist::error::ContractError as RoundWhitelistContractError;
use whitelist_types::{Round, RoundWhitelistQueryMsgs};

#[test]
fn test_remove_round() {
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
    let rounds = return_rounds();
    let res = app
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
    let round_whitelist_address = get_contract_address_from_res(res);
    // Try removing a round non admin
    let error = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(round_whitelist_address.clone()),
            &omniflix_round_whitelist::msg::ExecuteMsg::RemoveRound { round_index: 1 },
            &[],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<RoundWhitelistContractError>().unwrap();
    assert_eq!(error, &RoundWhitelistContractError::Unauthorized {});

    // Try removing a round out of index
    let error = app
        .execute_contract(
            admin.clone(),
            Addr::unchecked(round_whitelist_address.clone()),
            &omniflix_round_whitelist::msg::ExecuteMsg::RemoveRound { round_index: 3 },
            &[],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<RoundWhitelistContractError>().unwrap();
    assert_eq!(error, &RoundWhitelistContractError::RoundNotFound {});

    // Try removing a round which has started
    // First query the round 1 start time
    let round_data: Vec<(u32, Round)> = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: round_whitelist_address.clone(),
            msg: to_json_binary(&RoundWhitelistQueryMsgs::Rounds {}).unwrap(),
        }))
        .unwrap();
    let round_1_start_time = round_data[0].1.start_time;
    // Set time to round 1 start time
    app.set_block(BlockInfo {
        height: 1,
        time: round_1_start_time,
        chain_id: "test_1".to_string(),
    });
    // Try removing round 1
    let error = app
        .execute_contract(
            admin.clone(),
            Addr::unchecked(round_whitelist_address.clone()),
            &omniflix_round_whitelist::msg::ExecuteMsg::RemoveRound { round_index: 1 },
            &[],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<RoundWhitelistContractError>().unwrap();
    assert_eq!(error, &RoundWhitelistContractError::RoundAlreadyStarted {});

    // Reset block time
    app.set_block(BlockInfo {
        height: 1_000,
        time: Timestamp::from_nanos(1_000),
        chain_id: "test_1".to_string(),
    });
    // Now we can remove round 1
    let _res = app
        .execute_contract(
            admin.clone(),
            Addr::unchecked(round_whitelist_address.clone()),
            &omniflix_round_whitelist::msg::ExecuteMsg::RemoveRound { round_index: 1 },
            &[],
        )
        .unwrap();
    let round_data: Vec<(u32, Round)> = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: round_whitelist_address.clone(),
            msg: to_json_binary(&RoundWhitelistQueryMsgs::Rounds {}).unwrap(),
        }))
        .unwrap();
    assert_eq!(round_data.len(), 1);
    // Ensure that remaining round is round 2
    assert_eq!(round_data[0].1.start_time, Timestamp::from_nanos(4000));
    // Check that removed rounds index is 2
    assert_eq!(round_data[0].0, 2);
    // Add round
    let round = Round {
        start_time: Timestamp::from_nanos(6000),
        end_time: Timestamp::from_nanos(7000),
        addresses: vec![Addr::unchecked("collector".to_string())],
        round_per_address_limit: 1,
        mint_price: coin(1000000, "uflix"),
    };
    let _res = app
        .execute_contract(
            admin.clone(),
            Addr::unchecked(round_whitelist_address.clone()),
            &omniflix_round_whitelist::msg::ExecuteMsg::AddRound { round },
            &[],
        )
        .unwrap();
    let round_data: Vec<(u32, Round)> = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: round_whitelist_address.clone(),
            msg: to_json_binary(&RoundWhitelistQueryMsgs::Rounds {}).unwrap(),
        }))
        .unwrap();
    assert_eq!(round_data.len(), 2);
    // Ensure that round 2 start time is 6000
    assert_eq!(round_data[1].1.start_time, Timestamp::from_nanos(6000));
    // Check new round index, should be 3
    assert_eq!(round_data[1].0, 3);
}
