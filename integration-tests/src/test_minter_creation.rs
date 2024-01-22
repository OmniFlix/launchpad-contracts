#[cfg(test)]
mod test_minter_creation {

    use cosmwasm_std::{
        coin, to_json_binary, Addr, BlockInfo, Decimal, QueryRequest, StdError, Timestamp, Uint128,
        WasmQuery,
    };
    use cw_multi_test::Executor;
    use minter_types::Token;

    use minter_types::Config as MinterConfig;
    use minter_types::QueryMsg;

    use omniflix_minter_factory::msg::{
        ExecuteMsg as FactoryExecuteMsg, InstantiateMsg as FactoryInstantiateMsg,
    };

    use whitelist_types::{Round, RoundWhitelistQueryMsgs};

    use crate::utils::{get_minter_address_from_res, return_minter_instantiate_msg, return_rounds};

    use crate::{setup::setup, utils::query_onft_collection};
    use omniflix_minter::error::ContractError as MinterContractError;
    use omniflix_minter_factory::error::ContractError as MinterFactoryError;
    use omniflix_round_whitelist::error::ContractError as RoundWhitelistContractError;
    use omniflix_round_whitelist_factory::error::ContractError as RoundWhitelistFactoryContractError;

    #[test]
    fn test_minter_creation() {
        let (
            mut app,
            test_addresses,
            minter_factory_code_id,
            minter_code_id,
            _round_whitelist_factory_code_id,
            _round_whitelist_code_id,
            _open_edition_minter_factory_code_id,
            _open_edition_minter_code_id,
        ) = setup();
        let admin = test_addresses.admin;
        let creator = test_addresses.creator;
        let _collector = test_addresses.collector;

        let factory_inst_msg = FactoryInstantiateMsg {
            admin: Some(admin.to_string()),
            minter_creation_fee: coin(1000000, "uflix"),
            minter_code_id,
            fee_collector_address: admin.clone().into_string(),
            allowed_minter_mint_denoms: vec!["uflix".to_string()],
        };
        let factory_addr = app
            .instantiate_contract(
                minter_factory_code_id,
                admin.clone(),
                &factory_inst_msg,
                &[],
                "factory",
                None,
            )
            .unwrap();

        let minter_inst_msg = return_minter_instantiate_msg();
        let create_minter_msg = FactoryExecuteMsg::CreateMinter {
            msg: minter_inst_msg,
        };
        // Send no funds
        let error = app
            .execute_contract(
                creator.clone(),
                factory_addr.clone(),
                &create_minter_msg,
                &[],
            )
            .unwrap_err();

        let res = error.source().unwrap();
        let error = res.downcast_ref::<MinterFactoryError>().unwrap();
        assert_eq!(
            error,
            &MinterFactoryError::IncorrectFunds {
                expected: [coin(1000000, "uflix"), coin(1000000, "uflix")].to_vec(),
                actual: [].to_vec()
            }
        );
        // Send incorrect denom
        let error = app
            .execute_contract(
                creator.clone(),
                factory_addr.clone(),
                &create_minter_msg,
                &[coin(1000000, "diffirent_denom")],
            )
            .unwrap_err();

        let res = error.source().unwrap();
        let error = res.downcast_ref::<MinterFactoryError>().unwrap();
        assert_eq!(
            error,
            &MinterFactoryError::IncorrectFunds {
                expected: [coin(1000000, "uflix"), coin(1000000, "uflix")].to_vec(),
                actual: [coin(1000000, "diffirent_denom")].to_vec()
            }
        );
        // Send correct denom incorrect amount
        let error = app
            .execute_contract(
                creator.clone(),
                factory_addr.clone(),
                &create_minter_msg,
                &[coin(1000000, "uflix")],
            )
            .unwrap_err();

        let res = error.source().unwrap();
        let error = res.downcast_ref::<MinterFactoryError>().unwrap();
        assert_eq!(
            error,
            &MinterFactoryError::IncorrectFunds {
                expected: [coin(1000000, "uflix"), coin(1000000, "uflix")].to_vec(),
                actual: [coin(1000000, "uflix")].to_vec()
            }
        );

        // Send 0 num tokens
        let mut minter_inst_msg = return_minter_instantiate_msg();
        minter_inst_msg.init.num_tokens = 0;
        let create_minter_msg = FactoryExecuteMsg::CreateMinter {
            msg: minter_inst_msg,
        };
        let error = app
            .execute_contract(
                creator.clone(),
                factory_addr.clone(),
                &create_minter_msg,
                &[coin(2000000, "uflix")],
            )
            .unwrap_err();
        let res = error.source().unwrap().source().unwrap();
        let error = res.downcast_ref::<MinterContractError>().unwrap();
        assert_eq!(error, &MinterContractError::InvalidNumTokens {});

        // Send royalty ratio more than 100%
        let mut minter_inst_msg = return_minter_instantiate_msg();
        minter_inst_msg.init.royalty_ratio = "1.1".to_string();
        let create_minter_msg = FactoryExecuteMsg::CreateMinter {
            msg: minter_inst_msg,
        };
        let error = app
            .execute_contract(
                creator.clone(),
                factory_addr.clone(),
                &create_minter_msg,
                &[coin(2000000, "uflix")],
            )
            .unwrap_err();
        let res = error.source().unwrap().source().unwrap();
        let error = res.downcast_ref::<MinterContractError>().unwrap();
        assert_eq!(error, &MinterContractError::InvalidRoyaltyRatio {});

        // Send mint price 0
        let mut minter_inst_msg = return_minter_instantiate_msg();
        minter_inst_msg.init.mint_price = Uint128::zero();
        let create_minter_msg = FactoryExecuteMsg::CreateMinter {
            msg: minter_inst_msg,
        };
        let error = app
            .execute_contract(
                creator.clone(),
                factory_addr.clone(),
                &create_minter_msg,
                &[coin(2000000, "uflix")],
            )
            .unwrap_err();
        let res = error.source().unwrap().source().unwrap();
        let error = res.downcast_ref::<MinterContractError>().unwrap();
        assert_eq!(error, &MinterContractError::InvalidMintPrice {});

        // Incorrect start time
        let mut minter_inst_msg = return_minter_instantiate_msg();
        minter_inst_msg.init.start_time = Timestamp::from_nanos(1_000 - 1);
        let create_minter_msg = FactoryExecuteMsg::CreateMinter {
            msg: minter_inst_msg,
        };
        let error = app
            .execute_contract(
                creator.clone(),
                factory_addr.clone(),
                &create_minter_msg,
                &[coin(2000000, "uflix")],
            )
            .unwrap_err();
        let res = error.source().unwrap().source().unwrap();
        let error = res.downcast_ref::<MinterContractError>().unwrap();
        assert_eq!(error, &MinterContractError::InvalidStartTime {});

        // Incorrect end time
        let mut minter_inst_msg = return_minter_instantiate_msg();
        minter_inst_msg.init.end_time = Some(minter_inst_msg.init.start_time.minus_nanos(1));
        let create_minter_msg = FactoryExecuteMsg::CreateMinter {
            msg: minter_inst_msg,
        };
        let error = app
            .execute_contract(
                creator.clone(),
                factory_addr.clone(),
                &create_minter_msg,
                &[coin(2000000, "uflix")],
            )
            .unwrap_err();
        let res = error.source().unwrap().source().unwrap();
        let error = res.downcast_ref::<MinterContractError>().unwrap();
        assert_eq!(error, &MinterContractError::InvalidEndTime {});

        // Happy path
        let minter_inst_msg = return_minter_instantiate_msg();
        let create_minter_msg = FactoryExecuteMsg::CreateMinter {
            msg: minter_inst_msg,
        };
        // Query balance of factory admin before
        let query_res = app
            .wrap()
            .query_balance(admin.clone(), "uflix".to_string())
            .unwrap();
        let uflix_before = query_res.amount;

        let res = app
            .execute_contract(
                admin.clone(),
                factory_addr,
                &create_minter_msg,
                &[coin(2000000, "uflix")],
            )
            .unwrap();
        // Query balance of factory admin after
        let query_res = app
            .wrap()
            .query_balance(admin.clone(), "uflix".to_string())
            .unwrap();
        let uflix_after = query_res.amount;
        assert_eq!(uflix_before - uflix_after, Uint128::from(1000000u128));

        let minter_address = get_minter_address_from_res(res.clone());
        let storage = app.storage();
        let collection = query_onft_collection(storage, minter_address.clone());
        assert_eq!(collection.denom.clone().unwrap().name, "name".to_string());
        assert_eq!(collection.denom.unwrap().id, "id".to_string());

        // Query config
        let config_data: MinterConfig = app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: minter_address.clone(),
                msg: to_json_binary(&QueryMsg::Config {}).unwrap(),
            }))
            .unwrap();
        assert_eq!(config_data.per_address_limit, 1);
        assert_eq!(config_data.mint_price.denom, "uflix".to_string());
        assert_eq!(config_data.start_time, Timestamp::from_nanos(1000000000));
        assert_eq!(config_data.mint_price.amount, Uint128::from(1000000u128));
        assert_eq!(
            config_data.royalty_ratio,
            Decimal::from_ratio(1u128, 10u128)
        );
        assert_eq!(config_data.admin, Addr::unchecked("creator"));
        assert_eq!(
            config_data.payment_collector,
            Addr::unchecked("payment_collector")
        );

        // Query mintable tokens
        let mintable_tokens_data: Vec<Token> = app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: minter_address.clone(),
                msg: to_json_binary(&QueryMsg::MintableTokens {}).unwrap(),
            }))
            .unwrap();
        assert_eq!(mintable_tokens_data.len(), 1000);
        // This is not a proper check but I am making sure list is randomized and is not starting from 1
        assert_ne!(mintable_tokens_data[0].token_id, 1.to_string());

        // Check total tokens remaining
        let total_tokens_remaining_data: u32 = app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: minter_address.clone(),
                msg: to_json_binary(&QueryMsg::TotalTokens {}).unwrap(),
            }))
            .unwrap();
        assert_eq!(total_tokens_remaining_data, 1000);
    }

    #[test]
    fn test_whitelist_creation() {
        let (
            mut app,
            test_addresses,
            _minter_factory_code_id,
            _minter_code_id,
            round_whitelist_factory_code_id,
            round_whitelist_code_id,
            _open_edition_minter_factory_code_id,
            _open_edition_minter_code_id,
        ) = setup();
        let admin = test_addresses.admin;
        let creator = test_addresses.creator;
        let _collector = test_addresses.collector;

        let round_whitelist_factory_inst_msg =
            omniflix_round_whitelist_factory::msg::InstantiateMsg {
                admin: Some(admin.to_string()),
                fee_collector_address: admin.clone().into_string(),
                whitelist_code_id: round_whitelist_code_id,
                whitelist_creation_fee: coin(1000000, "uflix"),
            };
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
                        admin: Some(admin.to_string()),
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
            &RoundWhitelistFactoryContractError::MissingCreationFee {}
        );

        // Send more than fee amount
        let error = app
            .execute_contract(
                creator.clone(),
                round_whitelist_factory_addr.clone(),
                &omniflix_round_whitelist_factory::msg::ExecuteMsg::CreateWhitelist {
                    msg: whitelist_types::InstantiateMsg {
                        admin: Some(admin.to_string()),
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
                        admin: Some(admin.to_string()),
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
                        admin: Some(admin.to_string()),
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
                        admin: Some(admin.to_string()),
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
                        admin: Some(admin.to_string()),
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
                    admin: Some(admin.to_string()),
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
                        admin: Some(admin.to_string()),
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
        let round_whitelist_address = get_minter_address_from_res(res.clone());

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
        let rounds_data: Vec<Round> = app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: round_whitelist_address.clone(),
                msg: to_json_binary(&RoundWhitelistQueryMsgs::Rounds {}).unwrap(),
            }))
            .unwrap();
        assert_eq!(rounds_data.len(), 2);
        assert_eq!(rounds_data[0].start_time, Timestamp::from_nanos(2000));
        assert_eq!(rounds_data[0].end_time, Timestamp::from_nanos(3000));

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
        let round_data: Round = app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: round_whitelist_address.clone(),
                msg: to_json_binary(&RoundWhitelistQueryMsgs::ActiveRound {}).unwrap(),
            }))
            .unwrap();
        assert_eq!(round_data.start_time, Timestamp::from_nanos(2000));
        assert_eq!(round_data.end_time, Timestamp::from_nanos(3000));
    }
}
