#[cfg(test)]
mod test_round_whitelist_creation {

    use cosmwasm_std::{
        coin, to_json_binary, Addr, BlockInfo, QueryRequest, StdError, Timestamp, Uint128,
        WasmQuery,
    };
    use cw_multi_test::Executor;

    use whitelist_types::{Round, RoundWhitelistQueryMsgs};

    use crate::utils::{get_contract_address_from_res, return_factory_inst_message, return_rounds};

    use crate::setup::setup;

    use omniflix_round_whitelist::error::ContractError as RoundWhitelistContractError;
    use omniflix_round_whitelist_factory::error::ContractError as RoundWhitelistFactoryContractError;

    #[test]
    fn test_whitelist_creation() {
        let (
            mut app,
            test_addresses,
            _minter_factory_code_id,
            _minter_code_id,
            round_whitelist_factory_code_id,
            round_whitelist_code_id,
            _open_edition_minter_code_id,
            _open_edition_minter_factory_code_id,
        ) = setup();
        let admin = test_addresses.admin;
        let creator = test_addresses.creator;
        let _collector = test_addresses.collector;

        let round_whitelist_factory_inst_msg = return_factory_inst_message(round_whitelist_code_id);
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
                    msg: whitelist_types::InstantiateMsg {
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
                    msg: whitelist_types::InstantiateMsg {
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
                    msg: whitelist_types::InstantiateMsg {
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
                    msg: whitelist_types::InstantiateMsg {
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
                    msg: whitelist_types::InstantiateMsg {
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

        // 0 per address limit
        let mut rounds = return_rounds();
        rounds[0].round_per_address_limit = 0;
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
        let res = error.source().unwrap().source().unwrap();
        let error = res.downcast_ref::<RoundWhitelistContractError>().unwrap();
        assert_eq!(
            error,
            &RoundWhitelistContractError::InvalidPerAddressLimit {}
        );

        // Try instantiating without factory
        let rounds = return_rounds();

        // TODO - Find a way to essert Generic error without writing by hand
        let _error = app
            .instantiate_contract(
                round_whitelist_code_id,
                admin.clone(),
                &whitelist_types::InstantiateMsg {
                    admin: admin.to_string(),
                    rounds: rounds.clone(),
                },
                &[],
                "round_whitelist",
                None,
            )
            .unwrap_err();

        // Check factory admin balance before
        let query_res = app
            .wrap()
            .query_balance(admin.clone(), "uflix".to_string())
            .unwrap();
        let uflix_before = query_res.amount;

        // Happy path
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

        // Check factory admin balance after
        let query_res = app
            .wrap()
            .query_balance(admin.clone(), "uflix".to_string())
            .unwrap();
        let uflix_after = query_res.amount;
        assert_eq!(uflix_after - uflix_before, Uint128::from(1000000u128));
        // Too lazy to create one for whitelist it works
        let round_whitelist_address = get_contract_address_from_res(res.clone());

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
        let res: Result<Round, StdError> =
            app.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
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

        // Remove round which out of index
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

        // Try to remove round by non admin
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

        // Try removing round which has started
        app.set_block(BlockInfo {
            height: 1,
            time: Timestamp::from_nanos(2000 + 1),
            chain_id: "test_1".to_string(),
        });
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
        // Remove round
        let _res = app
            .execute_contract(
                admin.clone(),
                Addr::unchecked(round_whitelist_address.clone()),
                &omniflix_round_whitelist::msg::ExecuteMsg::RemoveRound { round_index: 2 },
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
        assert_eq!(round_data[0].1.start_time, Timestamp::from_nanos(2000));

        //Add Round Tests//
        // Try adding round which has started
        let round = Round {
            start_time: Timestamp::from_nanos(2000),
            end_time: Timestamp::from_nanos(3000),
            addresses: vec![Addr::unchecked("collector".to_string())],
            round_per_address_limit: 1,
            mint_price: coin(1000000, "uflix"),
        };
        let error = app
            .execute_contract(
                admin.clone(),
                Addr::unchecked(round_whitelist_address.clone()),
                &omniflix_round_whitelist::msg::ExecuteMsg::AddRound { round },
                &[],
            )
            .unwrap_err();
        let res = error.source().unwrap();
        let error = res.downcast_ref::<RoundWhitelistContractError>().unwrap();
        assert_eq!(error, &RoundWhitelistContractError::RoundAlreadyStarted {});

        // Try adding overlapped round
        let round = Round {
            start_time: Timestamp::from_nanos(2500),
            end_time: Timestamp::from_nanos(3500),
            addresses: vec![Addr::unchecked("collector".to_string())],
            round_per_address_limit: 1,
            mint_price: coin(1000000, "uflix"),
        };
        let error = app
            .execute_contract(
                admin.clone(),
                Addr::unchecked(round_whitelist_address.clone()),
                &omniflix_round_whitelist::msg::ExecuteMsg::AddRound { round },
                &[],
            )
            .unwrap_err();
        let res = error.source().unwrap();
        let error = res.downcast_ref::<RoundWhitelistContractError>().unwrap();
        assert_eq!(error, &RoundWhitelistContractError::RoundsOverlapped {});
        // Try adding proper round non admin
        let round = Round {
            start_time: Timestamp::from_nanos(6000),
            end_time: Timestamp::from_nanos(7000),
            addresses: vec![Addr::unchecked("collector".to_string())],
            round_per_address_limit: 1,
            mint_price: coin(1000000, "uflix"),
        };
        let error = app
            .execute_contract(
                creator.clone(),
                Addr::unchecked(round_whitelist_address.clone()),
                &omniflix_round_whitelist::msg::ExecuteMsg::AddRound { round },
                &[],
            )
            .unwrap_err();
        let res = error.source().unwrap();
        let error = res.downcast_ref::<RoundWhitelistContractError>().unwrap();
        assert_eq!(error, &RoundWhitelistContractError::Unauthorized {});
    }
}
