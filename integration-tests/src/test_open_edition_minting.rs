#[cfg(test)]
mod test_open_edition_minter_minting {

    use cosmwasm_std::{coin, coins, Addr, BlockInfo, Coin, Timestamp, Uint128};

    use cw_multi_test::{BankSudo, Executor, SudoMsg};
    use minter_types::{CollectionDetails, UserDetails};
    use omniflix_open_edition_minter_factory::msg::{
        ExecuteMsg as OpenEditionMinterFactoryExecuteMsg,
        InstantiateMsg as OpenEditionMinterFactoryInstantiateMsg, OpenEditionMinterCreateMsg,
        OpenEditionMinterInitExtention,
    };

    use crate::utils::{
        get_contract_address_from_res, return_open_edition_minter_inst_msg, return_rounds,
    };

    use crate::{setup::setup, utils::query_onft_collection};

    use omniflix_open_edition_minter::msg::ExecuteMsg as OpenEditionMinterExecuteMsg;

    use open_edition_minter_types::QueryMsg as OpenEditionMinterQueryMsg;

    use omniflix_open_edition_minter::error::ContractError as OpenEditionMinterError;

    #[test]
    fn test_open_edition_minting() {
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

        let res = app
            .execute_contract(
                creator.clone(),
                open_edition_minter_factory_address,
                &create_minter_msg,
                &[Coin::new(2000000, "uflix")],
            )
            .unwrap();
        let minter_address = get_contract_address_from_res(res);

        // Try minting before start time
        let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
        let res = app
            .execute_contract(
                collector.clone(),
                Addr::unchecked(minter_address.clone()),
                &mint_msg,
                &[Coin::new(1000000, "uflix")],
            )
            .unwrap_err();
        let err = res.source().unwrap();
        let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
        assert_eq!(
            error,
            &OpenEditionMinterError::MintingNotStarted {
                start_time: Timestamp::from_nanos(1_000_000_000),
                current_time: Timestamp::from_nanos(1_000)
            }
        );

        // Try minting with incorrect payment amount
        app.set_block(BlockInfo {
            chain_id: "test_1".to_string(),
            height: 1_000,
            time: Timestamp::from_nanos(1_000_000_000),
        });
        let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
        let res = app
            .execute_contract(
                collector.clone(),
                Addr::unchecked(minter_address.clone()),
                &mint_msg,
                &[Coin::new(1000000, "incorrect_denom")],
            )
            .unwrap_err();
        let err = res.source().unwrap();
        let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
        assert_eq!(
            error,
            &OpenEditionMinterError::PaymentError(cw_utils::PaymentError::MissingDenom(
                "uflix".to_string()
            ))
        );

        // Try minting with incorrect payment amount
        let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
        let res = app
            .execute_contract(
                collector.clone(),
                Addr::unchecked(minter_address.clone()),
                &mint_msg,
                &[Coin::new(1000000 - 1, "uflix")],
            )
            .unwrap_err();
        let err = res.source().unwrap();
        let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
        assert_eq!(
            error,
            &OpenEditionMinterError::IncorrectPaymentAmount {
                expected: Uint128::from(1000000u128),
                sent: Uint128::from(999999u128)
            }
        );
        // Minting after end time
        app.set_block(BlockInfo {
            chain_id: "test_1".to_string(),
            height: 1_000,
            time: Timestamp::from_nanos(2_000_000_000 + 1),
        });
        let mint_msg = OpenEditionMinterExecuteMsg::Mint {};

        let res = app
            .execute_contract(
                collector.clone(),
                Addr::unchecked(minter_address.clone()),
                &mint_msg,
                &[Coin::new(1000000, "uflix")],
            )
            .unwrap_err();
        let err = res.source().unwrap();
        let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
        assert_eq!(error, &OpenEditionMinterError::PublicMintingEnded {});

        // Set block time to valid minting time
        app.set_block(BlockInfo {
            chain_id: "test_1".to_string(),
            height: 1_000,
            time: Timestamp::from_nanos(1_000_000_000),
        });

        // Query uflix balance of creator before mint
        let creator_balance_before_mint: Uint128 = app
            .wrap()
            .query_balance(creator.to_string(), "uflix".to_string())
            .unwrap()
            .amount;
        // Mint
        let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
        let _res = app
            .execute_contract(
                collector.clone(),
                Addr::unchecked(minter_address.clone()),
                &mint_msg,
                &[Coin::new(1000000, "uflix")],
            )
            .unwrap();
        // Query uflix balance of creator after mint
        let creator_balance_after_mint: Uint128 = app
            .wrap()
            .query_balance(creator.to_string(), "uflix".to_string())
            .unwrap()
            .amount;
        // Check if creator got paid
        assert_eq!(
            creator_balance_after_mint,
            creator_balance_before_mint + Uint128::from(1000000u128)
        );

        // Query minter
        let query_msg = OpenEditionMinterQueryMsg::MintedTokens {
            address: collector.to_string(),
        };
        let res: UserDetails = app
            .wrap()
            .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
            .unwrap();
        assert_eq!(res.total_minted_count, 1);
        assert_eq!(res.minted_tokens[0].token_id, "1");

        // Query onft collection
        let collection = query_onft_collection(app.storage(), minter_address.clone());
        assert_eq!(collection.onfts.clone()[0].id, "1");
        assert_eq!(
            collection.onfts.clone()[0].metadata.clone().unwrap().name,
            "token_name # 1".to_string()
        );
        //     Query minter
        let query_msg = OpenEditionMinterQueryMsg::TokensRemaining {};
        let res: u32 = app
            .wrap()
            .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
            .unwrap();
        assert_eq!(res, 999);

        // Query minter
        let query_msg = OpenEditionMinterQueryMsg::TotalMintedCount {};
        let res: u32 = app
            .wrap()
            .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
            .unwrap();
        assert_eq!(res, 1);

        // Create a loop from 1 to 999 and mint every remaining token to receivers
        for i in 1..1000 {
            let collector = Addr::unchecked(format!("collector{}", i));
            // Mint money for collector
            app.sudo(SudoMsg::Bank(BankSudo::Mint {
                to_address: collector.to_string(),
                amount: coins(1000000, "uflix"),
            }))
            .unwrap();
            // Mint
            let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
            let _res = app
                .execute_contract(
                    collector.clone(),
                    Addr::unchecked(minter_address.clone()),
                    &mint_msg,
                    &[Coin::new(1000000, "uflix")],
                )
                .unwrap();

            // Query minter
            let query_msg = OpenEditionMinterQueryMsg::MintedTokens {
                address: collector.to_string(),
            };
            let res: UserDetails = app
                .wrap()
                .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
                .unwrap();
            assert_eq!(res.total_minted_count, 1);
        }

        // Query minter
        let query_msg = OpenEditionMinterQueryMsg::TokensRemaining {};
        let res: u32 = app
            .wrap()
            .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
            .unwrap();
        assert_eq!(res, 0);

        // Query minter
        let query_msg = OpenEditionMinterQueryMsg::TotalMintedCount {};
        let res: u32 = app
            .wrap()
            .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
            .unwrap();
        assert_eq!(res, 1000);

        // Try minting after all tokens are minted
        let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
        let res = app
            .execute_contract(
                collector.clone(),
                Addr::unchecked(minter_address.clone()),
                &mint_msg,
                &[Coin::new(1000000, "uflix")],
            )
            .unwrap_err();
        let err = res.source().unwrap();
        let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
        assert_eq!(error, &OpenEditionMinterError::NoTokensLeftToMint {});
    }

    #[test]
    fn test_open_edition_minter_private_minting() {
        let (
            mut app,
            test_addresses,
            _minter_factory_code_id,
            _minter_code_id,
            round_whitelist_factory_code_id,
            round_whitelist_code_id,
            open_edition_minter_factory_code_id,
            open_edition_minter_code_id,
        ) = setup();
        let admin = test_addresses.admin;
        let creator = test_addresses.creator;
        let collector = test_addresses.collector;

        // Instantiate the minter factory
        let open_edition_minter_factory_instantiate_msg = OpenEditionMinterFactoryInstantiateMsg {
            admin: Some(admin.to_string()),
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

        let round_whitelist_inst_msg = whitelist_types::InstantiateMsg {
            admin: Some(admin.to_string()),
            rounds: rounds.clone(),
        };
        let create_round_whitelist_msg =
            omniflix_round_whitelist_factory::msg::ExecuteMsg::CreateWhitelist {
                msg: round_whitelist_inst_msg,
            };
        let res = app
            .execute_contract(
                admin.clone(),
                round_whitelist_factory_addr,
                &create_round_whitelist_msg,
                &[coin(1000000, "uflix")],
            )
            .unwrap();
        let whitelist_address = get_contract_address_from_res(res);

        // Create a minter
        let mut open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
        open_edition_minter_instantiate_msg.init.whitelist_address =
            Some(whitelist_address.clone());
        let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateMinter {
            msg: open_edition_minter_instantiate_msg,
        };

        let res = app
            .execute_contract(
                creator.clone(),
                open_edition_minter_factory_address,
                &create_minter_msg,
                &[Coin::new(2000000, "uflix")],
            )
            .unwrap();
        let minter_address = get_contract_address_from_res(res);

        // Round 1 starts at 2000 and ends at 3000
        app.set_block(BlockInfo {
            chain_id: "test_1".to_string(),
            height: 1_000,
            time: Timestamp::from_nanos(2000 + 1),
        });

        // Mint for collector
        let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
        let _res = app
            .execute_contract(
                collector.clone(),
                Addr::unchecked(minter_address.clone()),
                &mint_msg.clone(),
                &[Coin::new(1000000, "diffirent_denom")],
            )
            .unwrap();

        // Try minting for creator
        let res = app
            .execute_contract(
                creator.clone(),
                Addr::unchecked(minter_address.clone()),
                &mint_msg.clone(),
                &[Coin::new(1000000, "diffirent_denom")],
            )
            .unwrap_err();
        let error = res.source().unwrap();
        let error = error.downcast_ref::<OpenEditionMinterError>().unwrap();
        assert_eq!(error, &OpenEditionMinterError::AddressNotWhitelisted {});

        app.set_block(BlockInfo {
            chain_id: "test_1".to_string(),
            height: 1_000,
            time: Timestamp::from_nanos(4000 + 1),
        });

        // Mint for creator
        let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
        let _res = app
            .execute_contract(
                creator.clone(),
                Addr::unchecked(minter_address.clone()),
                &mint_msg,
                &[Coin::new(1000000, "uflix")],
            )
            .unwrap();

        // Try minting for creator
        let res = app
            .execute_contract(
                creator.clone(),
                Addr::unchecked(minter_address.clone()),
                &mint_msg,
                &[Coin::new(1000000, "uflix")],
            )
            .unwrap_err();
        let error = res.source().unwrap();
        let error = error.downcast_ref::<OpenEditionMinterError>().unwrap();
        assert_eq!(error, &OpenEditionMinterError::AddressReachedMintLimit {});
    }

    // #[test]
    // fn test_open_edition_minter_with_editions() {
    //     let (
    //         mut app,
    //         test_addresses,
    //         _minter_factory_code_id,
    //         _minter_code_id,
    //         _round_whitelist_factory_code_id,
    //         _round_whitelist_code_id,
    //         open_edition_minter_factory_code_id,
    //         open_edition_minter_code_id,
    //     ) = setup();
    //     let admin = test_addresses.admin;
    //     let creator = test_addresses.creator;
    //     let collector = test_addresses.collector;

    //     // Instantiate the minter factory
    //     let open_edition_minter_factory_instantiate_msg = OpenEditionMinterFactoryInstantiateMsg {
    //         admin: Some(admin.to_string()),
    //         open_edition_minter_code_id,
    //         fee_collector_address: admin.to_string(),
    //         minter_creation_fee: coin(1000000, "uflix"),
    //     };

    //     let open_edition_minter_factory_address = app
    //         .instantiate_contract(
    //             open_edition_minter_factory_code_id,
    //             admin.clone(),
    //             &open_edition_minter_factory_instantiate_msg,
    //             &[],
    //             "Open Edition Minter Factory",
    //             None,
    //         )
    //         .unwrap();

    //     // Create a minter
    //     let collection_details = CollectionDetails {
    //         name: "Omniflix Community Editions".to_string(),
    //         description: "Omniflix Community Editions".to_string(),
    //         preview_uri: "preview_uri_1".to_string(),
    //         schema: "schema".to_string(),
    //         symbol: "symbol".to_string(),
    //         id: "id".to_string(),
    //         extensible: true,
    //         nsfw: false,
    //         base_uri: "base_uri_1".to_string(),
    //         uri: "uri_1".to_string(),
    //         uri_hash: "uri_hash_1".to_string(),
    //         data: "data_1".to_string(),
    //         token_name: "Omniflix Minter Community Edition 1".to_string(),
    //         transferable: true,
    //         royalty_receivers: None,
    //     };
    //     let init = OpenEditionMinterInitExtention {
    //         admin: "creator".to_string(),
    //         mint_price: Coin::new(1000000, "uflix"),
    //         start_time: Timestamp::from_nanos(1_000_000_000),
    //         end_time: Some(Timestamp::from_nanos(2_000_000_000)),
    //         per_address_limit: 3,
    //         royalty_ratio: "0.1".to_string(),
    //         payment_collector: Some("creator".to_string()),
    //         whitelist_address: None,
    //         token_limit: Some(1000),
    //     };
    //     let open_edition_minter_inst_msg = OpenEditionMinterCreateMsg {
    //         collection_details: collection_details,
    //         init: init,
    //     };
    //     let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateMinter {
    //         msg: open_edition_minter_inst_msg,
    //     };

    //     let res = app
    //         .execute_contract(
    //             creator.clone(),
    //             open_edition_minter_factory_address,
    //             &create_minter_msg,
    //             &[Coin::new(2000000, "uflix")],
    //         )
    //         .unwrap();
    //     let minter_address = get_contract_address_from_res(res);

    //     // Set block time to valid minting time
    //     app.set_block(BlockInfo {
    //         chain_id: "test_1".to_string(),
    //         height: 1_000,
    //         time: Timestamp::from_nanos(1_000_000_000),
    //     });

    //     // Mint for collector
    //     let mint_msg = OpenEditionMinterExecuteMsg::Mint { edition: None };
    //     let _res = app
    //         .execute_contract(
    //             collector.clone(),
    //             Addr::unchecked(minter_address.clone()),
    //             &mint_msg,
    //             &[Coin::new(1000000, "uflix")],
    //         )
    //         .unwrap();

    //     // Now collector owns a edition 1 nft
    //     // Query minter
    //     let query_msg = OpenEditionMinterQueryMsg::MintedTokens {
    //         address: collector.to_string(),
    //         edition: None,
    //     };
    //     let res: UserDetails = app
    //         .wrap()
    //         .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
    //         .unwrap();
    //     assert_eq!(res.total_minted_count, 1);
    //     assert_eq!(res.minted_tokens[0].token_id, "1");

    //     // Query MintedCount
    //     let query_msg = OpenEditionMinterQueryMsg::TotalMintedCount { edition: None };
    //     let res: u32 = app
    //         .wrap()
    //         .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
    //         .unwrap();
    //     assert_eq!(res, 1);

    //     let res: u32 = app
    //         .wrap()
    //         .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
    //         .unwrap();
    //     assert_eq!(res, 1);

    //     // Send new edition message
    //     let new_edition_msg = OpenEditionMinterExecuteMsg::NewEdition {
    //         whitelist_address: None,
    //         token_limit: Some(50),
    //         start_time: Timestamp::from_nanos(2_000_000_000),
    //         end_time: Some(Timestamp::from_nanos(3_000_000_000)),
    //         mint_price: Coin::new(2000000, "uflix"),
    //         royalty_ratio: "0.1".to_string(),
    //         token_name: "Omniflix Minter Community Edition 2".to_string(),
    //         description: "Omniflix Minter Community Edition 2".to_string(),
    //         base_uri: "base_uri_2".to_string(),
    //         preview_uri: "preview_uri_2".to_string(),
    //         uri_hash: "uri_hash_2".to_string(),
    //         transferable: true,
    //         extensible: true,
    //         nsfw: false,
    //         data: "data_2".to_string(),
    //     };

    //     let res = app
    //         .execute_contract(
    //             creator.clone(),
    //             Addr::unchecked(minter_address.clone()),
    //             &new_edition_msg,
    //             &[],
    //         )
    //         .unwrap();

    //     // Now most of the data is changed redo queries
    //     // Query minter
    //     let query_msg = OpenEditionMinterQueryMsg::MintedTokens {
    //         address: collector.to_string(),
    //         // If dont provide edition number it will use latest edition
    //         edition: None,
    //     };
    //     let res: UserDetails = app
    //         .wrap()
    //         .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
    //         .unwrap();

    //     assert_eq!(res.total_minted_count, 0);
    //     // Query with edition number
    //     let query_msg = OpenEditionMinterQueryMsg::MintedTokens {
    //         address: collector.to_string(),
    //         edition: Some(1),
    //     };
    //     let res: UserDetails = app
    //         .wrap()
    //         .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
    //         .unwrap();
    //     assert_eq!(res.total_minted_count, 1);

    //     // Query MintedCount
    //     let query_msg = OpenEditionMinterQueryMsg::TotalMintedCount { edition: None };
    //     let res: u32 = app
    //         .wrap()
    //         .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
    //         .unwrap();
    //     assert_eq!(res, 0);

    //     let query_msg = OpenEditionMinterQueryMsg::TotalMintedCount { edition: Some(1) };
    //     let res: u32 = app
    //         .wrap()
    //         .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
    //         .unwrap();
    //     assert_eq!(res, 1);

    //     // Try minting for collector 1 Should fail because minting is not started
    //     let mint_msg = OpenEditionMinterExecuteMsg::Mint { edition: None };
    //     let res = app
    //         .execute_contract(
    //             collector.clone(),
    //             Addr::unchecked(minter_address.clone()),
    //             &mint_msg,
    //             &[Coin::new(2000000, "uflix")],
    //         )
    //         .unwrap_err();
    //     let err = res.source().unwrap();
    //     let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
    //     assert_eq!(
    //         error,
    //         &OpenEditionMinterError::MintingNotStarted {
    //             start_time: Timestamp::from_nanos(2_000_000_000),
    //             current_time: Timestamp::from_nanos(1_000_000_000)
    //         }
    //     );

    //     // Set block time to valid minting time
    //     app.set_block(BlockInfo {
    //         chain_id: "test_1".to_string(),
    //         height: 1_000,
    //         time: Timestamp::from_nanos(2_000_000_000),
    //     });

    //     // Try minting for collector 1 with old payment amount Should fail because payment amount is changed
    //     let mint_msg = OpenEditionMinterExecuteMsg::Mint { edition: None };
    //     let res = app
    //         .execute_contract(
    //             collector.clone(),
    //             Addr::unchecked(minter_address.clone()),
    //             &mint_msg,
    //             &[Coin::new(1000000, "uflix")],
    //         )
    //         .unwrap_err();
    //     let err = res.source().unwrap();
    //     let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
    //     assert_eq!(
    //         error,
    //         &OpenEditionMinterError::IncorrectPaymentAmount {
    //             expected: Uint128::from(2000000u128),
    //             sent: Uint128::from(1000000u128)
    //         }
    //     );

    //     // Mint for collector
    //     let mint_msg = OpenEditionMinterExecuteMsg::Mint { edition: None };
    //     let _res = app
    //         .execute_contract(
    //             collector.clone(),
    //             Addr::unchecked(minter_address.clone()),
    //             &mint_msg,
    //             &[Coin::new(2000000, "uflix")],
    //         )
    //         .unwrap();
    //     // Now query onft and compare tokens
    //     let collection = query_onft_collection(app.storage(), minter_address.clone());
    //     assert_eq!(collection.onfts.clone()[0].id, "1");
    //     assert_eq!(
    //         collection.onfts.clone()[0].metadata.clone().unwrap().name,
    //         "Omniflix Minter Community Edition 1 # 1".to_string()
    //     );
    //     assert_eq!(collection.onfts.clone()[1].id, "2");
    //     assert_eq!(
    //         collection.onfts.clone()[1].metadata.clone().unwrap().name,
    //         "Omniflix Minter Community Edition 2 # 2".to_string()
    //     );
    //     assert_eq!(
    //         collection.onfts.clone()[1]
    //             .metadata
    //             .clone()
    //             .unwrap()
    //             .media_uri,
    //         "base_uri_2".to_string()
    //     );
    //     // Now we have two editions lets mint for first edition once again
    //     let mint_msg = OpenEditionMinterExecuteMsg::Mint { edition: Some(1) };
    //     let _res = app
    //         .execute_contract(
    //             collector.clone(),
    //             Addr::unchecked(minter_address.clone()),
    //             &mint_msg,
    //             &[Coin::new(1000000, "uflix")],
    //         )
    //         .unwrap();
    //     // Now query onft and compare tokens
    //     let collection = query_onft_collection(app.storage(), minter_address.clone());
    //     assert_eq!(collection.onfts.clone()[2].id, "3");
    //     assert_eq!(
    //         collection.onfts.clone()[2].metadata.clone().unwrap().name,
    //         "Omniflix Minter Community Edition 1 # 3".to_string()
    //     );
    // }
}
