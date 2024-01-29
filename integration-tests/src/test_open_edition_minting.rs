#[cfg(test)]
mod test_open_edition_minter_minting {

    use cosmwasm_std::{coin, coins, Addr, BlockInfo, Coin, Timestamp, Uint128};

    use cw_multi_test::{BankSudo, Executor, SudoMsg};
    use minter_types::UserDetails;
    use omniflix_open_edition_minter_factory::msg::{
        ExecuteMsg as OpenEditionMinterFactoryExecuteMsg,
        InstantiateMsg as OpenEditionMinterFactoryInstantiateMsg,
    };

    use crate::utils::{get_contract_address_from_res, return_open_edition_minter_inst_msg};

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
            "token_name".to_string()
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
}
