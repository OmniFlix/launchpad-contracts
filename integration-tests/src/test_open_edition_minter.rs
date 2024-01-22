#[cfg(test)]
mod test_open_edition_minter_creation {

    use cosmwasm_std::{
        coin, from_json, to_json_binary, Addr, BlockInfo, Coin, Decimal, QueryRequest, StdError,
        Timestamp, Uint128, WasmQuery,
    };

    use cw_multi_test::Executor;
    use minter_types::{CollectionDetails, Config};
    use omniflix_open_edition_minter_factory::msg::{
        ExecuteMsg as OpenEditionMinterFactoryExecuteMsg,
        InstantiateMsg as OpenEditionMinterFactoryInstantiateMsg,
    };

    use whitelist_types::{Round, RoundWhitelistQueryMsgs};

    use crate::utils::{
        get_minter_address_from_res, return_minter_instantiate_msg,
        return_open_edition_minter_inst_msg, return_rounds,
    };

    use crate::{setup::setup, utils::query_onft_collection};

    use omniflix_open_edition_minter::msg::ExecuteMsg as OpenEditionMinterExecuteMsg;

    use open_edition_minter_types::QueryMsg as OpenEditionMinterQueryMsg;

    use omniflix_open_edition_minter::error::ContractError as OpenEditionMinterError;

    use omniflix_open_edition_minter_factory::error::ContractError as OpenEditionMinterFactoryError;

    #[test]
    fn test_open_edition_minter_creation() {
        let (
            mut app,
            test_addresses,
            _minter_factory_code_id,
            _minter_code_id,
            _round_whitelist_factory_code_id,
            _round_whitelist_code_id,
            open_edition_minter_factory_code_id,
            open_edition_minter_code_id,
        ) = setup();
        let admin = test_addresses.admin;
        let creator = test_addresses.creator;
        let collector = test_addresses.collector;

        // Instantiate the minter factory
        let open_edition_minter_factory_instantiate_msg = OpenEditionMinterFactoryInstantiateMsg {
            admin: Some(admin.to_string()),
            allowed_minter_mint_denoms: vec!["uflix".to_string()],
            open_edition_minter_code_id,
            fee_collector_address: admin.to_string(),
            minter_creation_fee: coin(1000000, "uflix"),
        };

        let open_edition_minter_factory_address = app
            .instantiate_contract(
                open_edition_minter_factory_code_id,
                admin.clone(),
                &open_edition_minter_factory_instantiate_msg,
                &[],
                "Open Edition Minter Factory",
                None,
            )
            .unwrap();

        // Create a minter
        let open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
        let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateMinter {
            msg: open_edition_minter_instantiate_msg,
        };
        // Send no funds
        let res = app
            .execute_contract(
                creator.clone(),
                open_edition_minter_factory_address.clone(),
                &create_minter_msg,
                &[],
            )
            .unwrap_err();
        let err = res.source().unwrap();
        let error = err.downcast_ref::<OpenEditionMinterFactoryError>().unwrap();
        assert_eq!(
            OpenEditionMinterFactoryError::IncorrectFunds {
                expected: [
                    Coin {
                        denom: "uflix".to_string(),
                        amount: Uint128::from(1000000u128)
                    },
                    Coin {
                        denom: "uflix".to_string(),
                        amount: Uint128::from(1000000u128)
                    }
                ]
                .to_vec(),
                actual: vec![]
            },
            *error
        );

        // Send incorrect funds
        let res = app
            .execute_contract(
                creator.clone(),
                open_edition_minter_factory_address.clone(),
                &create_minter_msg,
                &[coin(1000000, "incorrect_denom")],
            )
            .unwrap_err();
        let err = res.source().unwrap();
        let error = err.downcast_ref::<OpenEditionMinterFactoryError>().unwrap();
        assert_eq!(
            OpenEditionMinterFactoryError::IncorrectFunds {
                expected: [
                    Coin {
                        denom: "uflix".to_string(),
                        amount: Uint128::from(1000000u128)
                    },
                    Coin {
                        denom: "uflix".to_string(),
                        amount: Uint128::from(1000000u128)
                    }
                ]
                .to_vec(),
                actual: vec![coin(1000000, "incorrect_denom")]
            },
            *error
        );

        // Send incorrect amount
        let res = app
            .execute_contract(
                creator.clone(),
                open_edition_minter_factory_address.clone(),
                &create_minter_msg,
                &[coin(1000000, "uflix")],
            )
            .unwrap_err();
        let err = res.source().unwrap();
        let error = err.downcast_ref::<OpenEditionMinterFactoryError>().unwrap();
        assert_eq!(
            OpenEditionMinterFactoryError::IncorrectFunds {
                expected: [
                    Coin {
                        denom: "uflix".to_string(),
                        amount: Uint128::from(1000000u128)
                    },
                    Coin {
                        denom: "uflix".to_string(),
                        amount: Uint128::from(1000000u128)
                    }
                ]
                .to_vec(),
                actual: vec![coin(1000000, "uflix")]
            },
            *error
        );

        // Send zero token limit
        let mut open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
        open_edition_minter_instantiate_msg.init.token_limit = Some(0);
        let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateMinter {
            msg: open_edition_minter_instantiate_msg,
        };
        let res = app
            .execute_contract(
                creator.clone(),
                open_edition_minter_factory_address.clone(),
                &create_minter_msg,
                &[coin(2000000, "uflix")],
            )
            .unwrap_err();

        let err = res.source().unwrap().source().unwrap();

        let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
        assert_eq!(OpenEditionMinterError::InvalidNumTokens {}, *error);

        // Send zero per address limit
        let mut open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
        open_edition_minter_instantiate_msg.init.per_address_limit = 0;
        let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateMinter {
            msg: open_edition_minter_instantiate_msg,
        };
        let res = app
            .execute_contract(
                creator.clone(),
                open_edition_minter_factory_address.clone(),
                &create_minter_msg,
                &[coin(2000000, "uflix")],
            )
            .unwrap_err();

        let err = res.source().unwrap().source().unwrap();

        let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
        assert_eq!(OpenEditionMinterError::PerAddressLimitZero {}, *error);

        // Send incorrect royalty ratio
        let mut open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
        open_edition_minter_instantiate_msg.init.royalty_ratio = "1.1".to_string();
        let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateMinter {
            msg: open_edition_minter_instantiate_msg,
        };
        let res = app
            .execute_contract(
                creator.clone(),
                open_edition_minter_factory_address.clone(),
                &create_minter_msg,
                &[coin(2000000, "uflix")],
            )
            .unwrap_err();

        let err = res.source().unwrap().source().unwrap();

        let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
        assert_eq!(OpenEditionMinterError::InvalidRoyaltyRatio {}, *error);

        // Send incorrect mint price this should not fail because mint price can be set to zero on open edition minter
        let mut open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
        open_edition_minter_instantiate_msg.init.mint_price = Uint128::zero();
        let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateMinter {
            msg: open_edition_minter_instantiate_msg,
        };
        let res = app
            .execute_contract(
                creator.clone(),
                open_edition_minter_factory_address.clone(),
                &create_minter_msg,
                &[coin(2000000, "uflix")],
            )
            .unwrap();

        // Send incorrect start time
        let mut open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
        open_edition_minter_instantiate_msg.init.start_time = Timestamp::from_nanos(0);
        let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateMinter {
            msg: open_edition_minter_instantiate_msg,
        };
        let res = app
            .execute_contract(
                creator.clone(),
                open_edition_minter_factory_address.clone(),
                &create_minter_msg,
                &[coin(2000000, "uflix")],
            )
            .unwrap_err();

        let err = res.source().unwrap().source().unwrap();

        let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
        assert_eq!(OpenEditionMinterError::InvalidStartTime {}, *error);

        // Check factory admin balance before happy path
        let query_res = app
            .wrap()
            .query_balance(admin.clone(), "uflix".to_string())
            .unwrap();
        let uflix_before = query_res.amount;

        // Create a minter
        let open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
        let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateMinter {
            msg: open_edition_minter_instantiate_msg,
        };
        let res = app
            .execute_contract(
                creator.clone(),
                open_edition_minter_factory_address.clone(),
                &create_minter_msg,
                &[coin(2000000, "uflix")],
            )
            .unwrap();
        let open_edition_minter_address = get_minter_address_from_res(res);

        // Check factory admin balance after happy path
        let query_res = app
            .wrap()
            .query_balance(admin.clone(), "uflix".to_string())
            .unwrap();
        let uflix_after = query_res.amount;
        // We are collecting fee as expected
        assert_eq!(uflix_after - uflix_before, Uint128::from(1000000u128));

        // Query the minter
        let query_msg = OpenEditionMinterQueryMsg::Config {};

        let config_res: Config = app
            .wrap()
            .query_wasm_smart(open_edition_minter_address.clone(), &query_msg)
            .unwrap();
        assert_eq!(
            config_res,
            Config {
                admin: Addr::unchecked(creator.clone()),
                payment_collector: Addr::unchecked(creator.clone()),
                end_time: Some(Timestamp::from_nanos(2_000_000_000)),
                start_time: Timestamp::from_nanos(1_000_000_000),
                mint_price: Coin {
                    denom: "uflix".to_string(),
                    amount: Uint128::from(1000000u128)
                },
                per_address_limit: 1,
                royalty_ratio: Decimal::percent(10),
                whitelist_address: None,
                token_limit: Some(1000),
            }
        );

        // Query the minter
        let query_msg = OpenEditionMinterQueryMsg::TokensRemaining {};

        let tokens_remaining_res: u32 = app
            .wrap()
            .query_wasm_smart(open_edition_minter_address.clone(), &query_msg)
            .unwrap();

        assert_eq!(tokens_remaining_res, 1000);

        // Query the minter
        let query_msg = OpenEditionMinterQueryMsg::TotalMintedCount {};

        let total_minted_count_res: u32 = app
            .wrap()
            .query_wasm_smart(open_edition_minter_address.clone(), &query_msg)
            .unwrap();

        assert_eq!(total_minted_count_res, 0);

        // Query the minter
        let query_msg = OpenEditionMinterQueryMsg::Collection {};

        let collection_res: CollectionDetails = app
            .wrap()
            .query_wasm_smart(open_edition_minter_address.clone(), &query_msg)
            .unwrap();

        assert_eq!(
            collection_res,
            CollectionDetails {
                name: "name".to_string(),
                description: "description".to_string(),
                preview_uri: "preview_uri".to_string(),
                schema: "schema".to_string(),
                symbol: "symbol".to_string(),
                id: "id".to_string(),
                extensible: true,
                nsfw: false,
                base_uri: "base_uri".to_string(),
                uri: "uri".to_string(),
                uri_hash: "uri_hash".to_string(),
                data: "data".to_string(),
            }
        );
    }
}
