#![cfg(test)]
use crate::helpers::mock_messages::factory_mock_messages::return_round_whitelist_factory_inst_message;
use crate::helpers::mock_messages::whitelist_mock_messages::return_round_configs;
use crate::helpers::setup::{setup, SetupResponse};
use crate::helpers::utils::get_contract_address_from_res;
use cosmwasm_std::{coin, Addr, Timestamp};

use cw_multi_test::Executor;
use omniflix_round_whitelist::error::ContractError as RoundWhitelistContractError;
use whitelist_types::{CreateWhitelistMsg, Round, RoundConfig};

#[test]
fn add_round() {
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
    // Try adding round with non admin
    let round = Round {
        start_time: Timestamp::from_nanos(1000),
        end_time: Timestamp::from_nanos(2000),
        round_per_address_limit: 1,
        mint_price: coin(1000000, "uflix"),
    };
    let round_members = vec!["collector".to_string()];
    let round_config = RoundConfig {
        round,
        members: round_members.clone(),
    };

    let error = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(round_whitelist_address.clone()),
            &omniflix_round_whitelist::msg::ExecuteMsg::AddRound { round_config },
            &[],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<RoundWhitelistContractError>().unwrap();
    assert_eq!(error, &RoundWhitelistContractError::Unauthorized {});

    // Try adding round which has started
    let round = Round {
        start_time: Timestamp::from_nanos(500),
        end_time: Timestamp::from_nanos(1800),
        round_per_address_limit: 1,
        mint_price: coin(1000000, "uflix"),
    };
    let round_members = vec!["collector".to_string()];
    let round_config = RoundConfig {
        round,
        members: round_members.clone(),
    };

    let error = app
        .execute_contract(
            admin.clone(),
            Addr::unchecked(round_whitelist_address.clone()),
            &omniflix_round_whitelist::msg::ExecuteMsg::AddRound { round_config },
            &[],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<RoundWhitelistContractError>().unwrap();
    assert_eq!(error, &RoundWhitelistContractError::RoundAlreadyStarted {});

    // Try adding overlapped round
    let overlapping_round = Round {
        start_time: Timestamp::from_nanos(2500),
        end_time: Timestamp::from_nanos(3500),
        round_per_address_limit: 1,
        mint_price: coin(1000000, "uflix"),
    };
    let round_members = vec!["collector".to_string()];
    let round_config = RoundConfig {
        round: overlapping_round,
        members: round_members.clone(),
    };

    let error = app
        .execute_contract(
            admin.clone(),
            Addr::unchecked(round_whitelist_address.clone()),
            &omniflix_round_whitelist::msg::ExecuteMsg::AddRound {
                round_config: round_config.clone(),
            },
            &[],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<RoundWhitelistContractError>().unwrap();
    assert_eq!(error, &RoundWhitelistContractError::RoundsOverlapped {});

    // Try adding invalid end time
    let invalid_end_time_round = Round {
        start_time: Timestamp::from_nanos(4000),
        end_time: Timestamp::from_nanos(3000),
        round_per_address_limit: 1,
        mint_price: coin(1000000, "uflix"),
    };
    let round_members = vec!["collector".to_string()];
    let round_config = RoundConfig {
        round: invalid_end_time_round,
        members: round_members.clone(),
    };

    let error = app
        .execute_contract(
            admin.clone(),
            Addr::unchecked(round_whitelist_address.clone()),
            &omniflix_round_whitelist::msg::ExecuteMsg::AddRound {
                round_config: round_config.clone(),
            },
            &[],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<RoundWhitelistContractError>().unwrap();
    assert_eq!(error, &RoundWhitelistContractError::InvalidEndTime {});

    // Try adding empty addresses
    let empty_addresses_round = Round {
        start_time: Timestamp::from_nanos(5000),
        end_time: Timestamp::from_nanos(6000),
        round_per_address_limit: 1,
        mint_price: coin(1000000, "uflix"),
    };
    let empty_addresses = vec![];
    let emty_addresses_round_config = RoundConfig {
        round: empty_addresses_round,
        members: empty_addresses.clone(),
    };
    let error = app
        .execute_contract(
            admin.clone(),
            Addr::unchecked(round_whitelist_address.clone()),
            &omniflix_round_whitelist::msg::ExecuteMsg::AddRound {
                round_config: emty_addresses_round_config.clone(),
            },
            &[],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<RoundWhitelistContractError>().unwrap();
    assert_eq!(error, &RoundWhitelistContractError::EmptyAddressList {});

    // Try adding invalid per address limit
    let invalid_per_address_limit_round = Round {
        start_time: Timestamp::from_nanos(4000),
        end_time: Timestamp::from_nanos(5000),
        round_per_address_limit: 0,
        mint_price: coin(1000000, "uflix"),
    };
    let round_members = vec!["collector".to_string()];
    let round_config = RoundConfig {
        round: invalid_per_address_limit_round,
        members: round_members.clone(),
    };
    let error = app
        .execute_contract(
            admin.clone(),
            Addr::unchecked(round_whitelist_address.clone()),
            &omniflix_round_whitelist::msg::ExecuteMsg::AddRound {
                round_config: round_config.clone(),
            },
            &[],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<RoundWhitelistContractError>().unwrap();
    assert_eq!(
        error,
        &RoundWhitelistContractError::InvalidPerAddressLimit {}
    );
}
