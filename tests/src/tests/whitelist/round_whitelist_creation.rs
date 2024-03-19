#![cfg(test)]
use cosmwasm_std::{coin, Addr, Timestamp, Uint128};

use crate::helpers::mock_messages::factory_mock_messages::return_round_whitelist_factory_inst_message;
use crate::helpers::mock_messages::whitelist_mock_messages::return_rounds;
use crate::helpers::setup::{setup, SetupResponse};
use crate::helpers::utils::get_contract_address_from_res;

use cw_multi_test::Executor;
use omniflix_round_whitelist::error::ContractError as RoundWhitelistContractError;
use omniflix_round_whitelist_factory::error::ContractError as RoundWhitelistFactoryContractError;
use whitelist_types::{CreateWhitelistMsg, RoundWhitelistQueryMsgs};

#[test]
fn whitelist_creation() {
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

    // Send wrong fee amount
    let error = app
        .execute_contract(
            creator.clone(),
            round_whitelist_factory_addr.clone(),
            &omniflix_round_whitelist_factory::msg::ExecuteMsg::CreateWhitelist {
                msg: CreateWhitelistMsg {
                    admin: admin.to_string(),
                    rounds: rounds.clone(),
                },
            },
            &[coin(1000, "diffirent_denom")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res
        .downcast_ref::<RoundWhitelistFactoryContractError>()
        .unwrap();
    assert_eq!(
        error,
        &RoundWhitelistFactoryContractError::PaymentError(cw_utils::PaymentError::ExtraDenom(
            "diffirent_denom".to_string()
        ))
    );

    // Send more than fee amount
    let error = app
        .execute_contract(
            creator.clone(),
            round_whitelist_factory_addr.clone(),
            &omniflix_round_whitelist_factory::msg::ExecuteMsg::CreateWhitelist {
                msg: CreateWhitelistMsg {
                    admin: admin.to_string(),
                    rounds: rounds.clone(),
                },
            },
            &[coin(1000001, "uflix")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res
        .downcast_ref::<RoundWhitelistFactoryContractError>()
        .unwrap();
    assert_eq!(
        error,
        &RoundWhitelistFactoryContractError::MissingCreationFee {}
    );

    // Invalid start time for first round
    let mut rounds = return_rounds();
    rounds[0].start_time = Timestamp::from_nanos(1000 - 1);
    let error = app
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
        .unwrap_err();
    let res = error.source().unwrap().source().unwrap();
    let error = res.downcast_ref::<RoundWhitelistContractError>().unwrap();
    assert_eq!(error, &RoundWhitelistContractError::RoundAlreadyStarted {});

    // Invalid end time for first round
    let mut rounds = return_rounds();
    rounds[0].start_time = Timestamp::from_nanos(2000);
    rounds[0].end_time = Timestamp::from_nanos(2000 - 1);
    let error = app
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
        .unwrap_err();
    let res = error.source().unwrap().source().unwrap();
    let error = res.downcast_ref::<RoundWhitelistContractError>().unwrap();
    assert_eq!(error, &RoundWhitelistContractError::InvalidEndTime {});

    // 0 per address limit
    let mut rounds = return_rounds();
    rounds[0].round_per_address_limit = 0;
    let error = app
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
        .unwrap_err();
    let res = error.source().unwrap().source().unwrap();
    let error = res.downcast_ref::<RoundWhitelistContractError>().unwrap();
    assert_eq!(
        error,
        &RoundWhitelistContractError::InvalidPerAddressLimit {}
    );

    // Try instantiating without factory
    let rounds = return_rounds();

    let _error = app
        .instantiate_contract(
            round_whitelist_code_id,
            admin.clone(),
            &CreateWhitelistMsg {
                admin: admin.to_string(),
                rounds: rounds.clone(),
            },
            &[],
            "round_whitelist",
            None,
        )
        .unwrap_err();
    // Overlapping rounds
    let mut rounds = return_rounds();
    rounds[0].start_time = Timestamp::from_nanos(2000);
    rounds[0].end_time = Timestamp::from_nanos(3000);
    rounds[1].start_time = Timestamp::from_nanos(2500);
    rounds[1].end_time = Timestamp::from_nanos(3500);
    let error = app
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
        .unwrap_err();
    let res = error.source().unwrap().source().unwrap();
    let error = res.downcast_ref::<RoundWhitelistContractError>().unwrap();
    assert_eq!(error, &RoundWhitelistContractError::RoundsOverlapped {});

    // Send more than 5000 diffirent members for a round
    let mut rounds = return_rounds();
    for i in 0..5001 {
        let address = Addr::unchecked(format!("collector{}", i));
        rounds[0].addresses.push(address);
    }

    let error = app
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
        .unwrap_err();
    let res = error.source().unwrap().source().unwrap();
    let error = res.downcast_ref::<RoundWhitelistContractError>().unwrap();
    assert_eq!(
        error,
        &RoundWhitelistContractError::WhitelistMemberLimitExceeded {}
    );
    // Send more than 5000 Same members for a round should not fail
    let mut rounds = return_rounds();
    for _i in 0..5001 {
        let address = Addr::unchecked("collector");
        rounds[0].addresses.push(address);
    }

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
    assert_eq!(members.len(), 1);

    // Check factory admin balance before
    let query_res = app
        .wrap()
        .query_balance(admin.clone(), "uflix".to_string())
        .unwrap();
    let uflix_before = query_res.amount;

    // Happy path
    let rounds = return_rounds();
    let _res = app
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

    // Check factory admin balance after
    let query_res = app
        .wrap()
        .query_balance(admin.clone(), "uflix".to_string())
        .unwrap();
    let uflix_after = query_res.amount;
    assert_eq!(uflix_after - uflix_before, Uint128::from(1000000u128));
}
