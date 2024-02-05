#[cfg(test)]
mod scenarios {

    use std::ops::Add;

    use cosmwasm_std::{
        coin, coins, to_json_binary, Addr, BlockInfo, QueryRequest, Timestamp, Uint128, WasmQuery,
    };
    use cw_multi_test::{BankSudo, Executor, SudoMsg};
    use minter_types::CollectionDetails;
    use minter_types::Token;
    use minter_types::UserDetails;

    use minter_types::QueryMsg;
    use omniflix_minter::msg::ExecuteMsg as MinterExecuteMsg;
    use omniflix_minter_factory::msg::CreateMinterMsg;
    use omniflix_minter_factory::msg::MinterInitExtention;
    use omniflix_minter_factory::msg::{
        ExecuteMsg as FactoryExecuteMsg, InstantiateMsg as FactoryInstantiateMsg,
    };
    use omniflix_open_edition_minter::error;
    use omniflix_round_whitelist::msg::ExecuteMsg as RoundWhitelistExecuteMsg;
    use omniflix_round_whitelist::round;
    use whitelist_types::Round;
    use whitelist_types::RoundWhitelistQueryMsgs;

    use crate::utils::{
        get_contract_address_from_res, mint_to_address, return_minter_instantiate_msg,
        return_rounds,
    };

    use crate::{setup::setup, utils::query_onft_collection};
    use omniflix_minter::error::ContractError as MinterContractError;

    use omniflix_round_whitelist::error::ContractError as RoundWhitelistContractError;

    #[test]
    fn test_scenario_1() {
        // Scenario 1:
        // 1. Creator creates a new round whitelist contract with 3 rounds
        // Rounds: 1, 2, 3
        // Round_1:
        // Round{
        //     start_time: 1_000_000,
        //     end_time: 2_000_000,
        //     round_per_address_limit: 1,
        //     addresses: ["collector"+ 1...25],
        //     round_price : 1_000_000 uflix
        // }
        // Round_2:
        // Round{
        //     start_time: 2_000_000,
        //     end_time: 3_000_000,
        //     round_per_address_limit: 1,
        //     addresses: ["collector"+ 1 ... 25],
        //     round_price : 2_000_000 ibc_atom
        // }
        // Round_3:
        // Round{
        //     start_time: 3_000_000,
        //     end_time: 4_000_000,
        //     round_per_address_limit: 1,
        //     addresses: ["collector"+ 26 ... 50],
        //     round_price : 3_000_000 ibc_atom
        // }

        // 2. Creator creates 2 minter contracts sending same whitelist address
        // Minter_1:
        // {
        // public_mint_start_time: 1_000_000_000,
        // public_mint_end_time: 2_000_000_000,
        // public_mint_limit:1,
        // public_mint_price: 5_000_000 uflix,
        // supply: 100,
        //   }
        // Minter_2:
        // {
        // public_mint_start_time: 2_000_000_000,
        // public_mint_end_time: 3_000_000_000,
        // public_mint_limit: 100,
        // public_mint_price: 10_000_000 uflix,
        // supply: 100,
        //}
        // 3. Collector_1 buys 1 NFT from Minter_1 during round 1
        // 4. Collector_1 buys 1 NFT from Minter_2 during round 1
        // Expected scenario is that Collector_1 should be able to buy 1 NFT from Minter_1 and 1 NFT from Minter_2
        // Round whitelist saves collectors minted NFTs from diffirent minter contracts seperately and they dont affect each other
        // This event proves that because round 1 mint limit is 1 and collector_1 already bought 1 NFT from minter_1,
        // he cant buy another NFT from minter_1 during first round but he can buy 1 NFT from minter_2
        // 5. Collector_1 waits for round 2 to buy 1 NFT from Minter_1
        // In total Collector_1 has 2 NFTs from minter_1 and 1 NFT from minter_2
        // This proves that public mint limit is not affecting private mints because public mint limit is 1 but collector_1 has 2 NFTs from minter_1
        // 6. By the time round 3 starts, collector_1 realizes that he is not in the whitelist for round 3
        // Creator add collector_1 to round 3 whitelist addresses
        // 7. Collector_1 buys 1 NFT from Minter_1 during round 3
        // 8. When round 3 ends, minter_1 had only minted 3 NFTs so creator decides to add 1 more round
        // Round_4:
        // Round{
        //     start_time: 4_000_000,
        //     end_time: 5_000_000,
        //     round_per_address_limit: 1,
        //     addresses: ["collector"+ 1 ... 100],
        //     round_price : 200_000 uflix
        // }
        // 9. Collector_1 buys 1 NFT from Minter_1 during round 4
        // 10. Collector_1 has 4 NFTs from minter_1 and 1 NFT from minter_2
        // 11. Creator waits for public mint to start and buys 1 NFT from Minter_1
        // 12. Creator can not buy another NFT from Minter_1 because public mint limit is 1

        let (
            mut app,
            test_addresses,
            minter_factory_code_id,
            minter_code_id,
            round_whitelist_factory_code_id,
            round_whitelist_code_id,
            _open_edition_minter_code_id,
            _open_edition_minter_factory_code_id,
        ) = setup();
        let admin = test_addresses.admin;
        let creator = test_addresses.creator;
        let _collector = test_addresses.collector;

        let factory_inst_msg = FactoryInstantiateMsg {
            admin: Some(admin.to_string()),
            minter_creation_fee: coin(1000000, "uflix"),
            minter_code_id,
            fee_collector_address: admin.clone().into_string(),
        };
        let minter_factory_addr = app
            .instantiate_contract(
                minter_factory_code_id,
                admin.clone(),
                &factory_inst_msg,
                &[],
                "factory",
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

        let rounds: Vec<Round> = [
            Round {
                start_time: Timestamp::from_nanos(1_000_000),
                end_time: Timestamp::from_nanos(2_000_000),
                round_per_address_limit: 1,
                addresses: (1..=25)
                    .map(|i| Addr::unchecked(format!("collector{}", i)))
                    .collect::<Vec<Addr>>(),
                mint_price: coin(1_000_000, "uflix"),
            },
            Round {
                start_time: Timestamp::from_nanos(2_000_000),
                end_time: Timestamp::from_nanos(3_000_000),
                round_per_address_limit: 1,
                addresses: (1..=25)
                    .map(|i| Addr::unchecked(format!("collector{}", i)))
                    .collect::<Vec<Addr>>(),
                mint_price: coin(2_000_000, "ibc_atom"),
            },
            Round {
                start_time: Timestamp::from_nanos(3_000_000),
                end_time: Timestamp::from_nanos(4_000_000),
                round_per_address_limit: 1,
                addresses: (26..=50)
                    .map(|i| Addr::unchecked(format!("collector{}", i)))
                    .collect::<Vec<Addr>>(),
                mint_price: coin(3_000_000, "ibc_atom"),
            },
        ]
        .to_vec();

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
                creator.clone(),
                round_whitelist_factory_addr.clone(),
                &create_round_whitelist_msg,
                &[coin(1000000, "uflix")],
            )
            .unwrap();
        let round_whitelist_addr = get_contract_address_from_res(res);

        let minter_1_inst_message = CreateMinterMsg {
            collection_details: CollectionDetails {
                name: "Test_collection_1".to_string(),
                description: "description".to_string(),
                preview_uri: "preview_uri".to_string(),
                schema: "schema".to_string(),
                symbol: "symbol".to_string(),
                id: "test1".to_string(),
                extensible: true,
                nsfw: false,
                base_uri: "base_uri".to_string(),
                uri: "uri".to_string(),
                uri_hash: "uri_hash".to_string(),
                data: "data".to_string(),
                token_name: "token_name".to_string(),
                transferable: true,
                royalty_receivers: None,
            },
            init: MinterInitExtention {
                admin: creator.to_string(),
                mint_price: coin(5_000_000, "uflix"),
                start_time: Timestamp::from_nanos(1_000_000_000),
                end_time: Some(Timestamp::from_nanos(2_000_000_000)),
                per_address_limit: 1,
                royalty_ratio: "0.1".to_string(),
                payment_collector: Some(creator.to_string()),
                whitelist_address: Some(round_whitelist_addr.clone()),
                num_tokens: 100,
            },
        };
        let mut minter_2_inst_msg = minter_1_inst_message.clone();
        minter_2_inst_msg.init.mint_price = coin(10_000_000, "uflix");
        minter_2_inst_msg.init.start_time = Timestamp::from_nanos(2_000_000_000);
        minter_2_inst_msg.init.end_time = Some(Timestamp::from_nanos(3_000_000_000));
        minter_2_inst_msg.init.per_address_limit = 100;
        minter_2_inst_msg.init.num_tokens = 100;
        minter_2_inst_msg.collection_details.id = "test2".to_string();
        minter_2_inst_msg.collection_details.name = "Test_collection_2".to_string();

        // Instantiate minter_1
        let res = app
            .execute_contract(
                creator.clone(),
                minter_factory_addr.clone(),
                &FactoryExecuteMsg::CreateMinter {
                    msg: minter_1_inst_message.clone(),
                },
                &[coin(2000000, "uflix")],
            )
            .unwrap();
        let minter_1_addr = get_contract_address_from_res(res);

        // Instantiate minter_2
        let res = app
            .execute_contract(
                creator.clone(),
                minter_factory_addr.clone(),
                &FactoryExecuteMsg::CreateMinter {
                    msg: minter_2_inst_msg.clone(),
                },
                &[coin(2000000, "uflix")],
            )
            .unwrap();
        let minter_2_addr = get_contract_address_from_res(res);

        // Collector_1 buys 1 NFT from Minter_1 during round 1
        // Price is 1_000_000 uflix
        let collector_1 = Addr::unchecked("collector1");
        // Mint flix to collector_1
        mint_to_address(
            &mut app,
            collector_1.to_string(),
            coins(1000000 + 1000000, "uflix"),
        );
        app.set_block(BlockInfo {
            chain_id: "test_1".to_string(),
            height: 1_000,
            time: Timestamp::from_nanos(1_000_000 + 1),
        });
        let res = app
            .execute_contract(
                collector_1.clone(),
                Addr::unchecked(minter_1_addr.clone()),
                &MinterExecuteMsg::Mint {},
                &[coin(1000000, "uflix")],
            )
            .unwrap();

        // Collector_1 buys 1 NFT from Minter_2 during round 1
        // Price is round 1 price and its 1_000_000 uflix
        let res = app
            .execute_contract(
                collector_1.clone(),
                Addr::unchecked(minter_2_addr.clone()),
                &MinterExecuteMsg::Mint {},
                &[coin(1000000, "uflix")],
            )
            .unwrap();

        // Collector 1 depleted his funds
        let balance = app
            .wrap()
            .query_balance(collector_1.clone(), "uflix")
            .unwrap();
        assert_eq!(balance.amount, Uint128::zero());

        // Query Onft
        let collection_1 = query_onft_collection(app.storage(), minter_1_addr.clone());
        assert_eq!(collection_1.onfts.len(), 1);
        let collection_2 = query_onft_collection(app.storage(), minter_2_addr.clone());
        assert_eq!(collection_2.onfts.len(), 1);

        // Collector one tries to buy another NFT from minter_1 during round 1
        // This should fail because he already bought 1 NFT from minter_1 during round 1
        // Mint flix to collector_1
        mint_to_address(&mut app, collector_1.to_string(), coins(1000000, "uflix"));
        let res = app
            .execute_contract(
                collector_1.clone(),
                Addr::unchecked(minter_1_addr.clone()),
                &MinterExecuteMsg::Mint {},
                &[coin(1000000, "uflix")],
            )
            .unwrap_err();
        let err = res.source().unwrap().source().unwrap();
        let error = err.downcast_ref::<RoundWhitelistContractError>().unwrap();
        assert_eq!(
            error,
            &RoundWhitelistContractError::RoundReachedMintLimit {}
        );

        // Collector_1 waits for round 2 to buy 1 NFT from Minter_1
        // Price is 2_000_000 ibc_atom
        app.set_block(BlockInfo {
            chain_id: "test_1".to_string(),
            height: 1_000,
            time: Timestamp::from_nanos(2_000_000 + 1),
        });
        // Mint ibc_atom to collector_1
        mint_to_address(
            &mut app,
            collector_1.to_string(),
            coins(2000000, "ibc_atom"),
        );
        let res = app
            .execute_contract(
                collector_1.clone(),
                Addr::unchecked(minter_1_addr.clone()),
                &MinterExecuteMsg::Mint {},
                &[coin(2000000, "ibc_atom")],
            )
            .unwrap();

        // Collector_1 depleted his funds
        let balance = app
            .wrap()
            .query_balance(collector_1.clone(), "ibc_atom")
            .unwrap();
        assert_eq!(balance.amount, Uint128::zero());

        // Query Onft
        let collection_1 = query_onft_collection(app.storage(), minter_1_addr.clone());
        assert_eq!(collection_1.onfts.len(), 2);

        // Wait for round 3 to start
        app.set_block(BlockInfo {
            chain_id: "test_1".to_string(),
            height: 1_000,
            time: Timestamp::from_nanos(3_000_000 + 1),
        });
        // Mint ibc_atom to collector_1
        mint_to_address(
            &mut app,
            collector_1.to_string(),
            coins(3000000, "ibc_atom"),
        );
        // Collector_1 buys 1 NFT from Minter_1 during round 3
        // Price is 3_000_000 ibc_atom
        // Should fail because collector_1 is not in the whitelist for round 3
        let res = app
            .execute_contract(
                collector_1.clone(),
                Addr::unchecked(minter_1_addr.clone()),
                &MinterExecuteMsg::Mint {},
                &[coin(3000000, "ibc_atom")],
            )
            .unwrap_err();
        let err = res.source().unwrap();
        let error = err.downcast_ref::<MinterContractError>().unwrap();
        assert_eq!(error, &MinterContractError::AddressNotWhitelisted {});

        // Add collector_1 to round 3 whitelist addresses
        let rounds: Vec<(u32, Round)> = app
            .wrap()
            .query_wasm_smart(
                round_whitelist_addr.clone(),
                &RoundWhitelistQueryMsgs::Rounds {},
            )
            .unwrap();

        // Found the index of round 3(Spoiler alert: its 3)
        let index = rounds
            .iter()
            .find(|(_index, round)| round.start_time == Timestamp::from_nanos(3_000_000))
            .unwrap()
            .0;

        // Add collector_1 to round 3 whitelist addresses
        let res = app
            .execute_contract(
                admin.clone(),
                Addr::unchecked(round_whitelist_addr.clone()),
                &RoundWhitelistExecuteMsg::AddMembers {
                    address: vec![collector_1.clone().to_string()],
                    round_index: index as u32,
                },
                &[],
            )
            .unwrap();

        // Collector_1 buys 1 NFT from Minter_1 during round 3
        let _res = app
            .execute_contract(
                collector_1.clone(),
                Addr::unchecked(minter_1_addr.clone()),
                &MinterExecuteMsg::Mint {},
                &[coin(3000000, "ibc_atom")],
            )
            .unwrap();

        // Query Onft
        let collection_1 = query_onft_collection(app.storage(), minter_1_addr.clone());
        assert_eq!(collection_1.onfts.len(), 3);

        // Creator adds 1 more round
        let round_4 = Round {
            start_time: Timestamp::from_nanos(4_000_000),
            end_time: Timestamp::from_nanos(5_000_000),
            round_per_address_limit: 1,
            addresses: (1..=100)
                .map(|i| Addr::unchecked(format!("collector{}", i)))
                .collect::<Vec<Addr>>(),
            mint_price: coin(200_000, "uflix"),
        };
        let res = app
            .execute_contract(
                admin.clone(),
                Addr::unchecked(round_whitelist_addr.clone()),
                &RoundWhitelistExecuteMsg::AddRound { round: round_4 },
                &[],
            )
            .unwrap();
        // Query rounds
        let rounds: Vec<(u32, Round)> = app
            .wrap()
            .query_wasm_smart(
                round_whitelist_addr.clone(),
                &RoundWhitelistQueryMsgs::Rounds {},
            )
            .unwrap();
        assert_eq!(rounds.len(), 4);

        // Wait for round 4 to start
        app.set_block(BlockInfo {
            chain_id: "test_1".to_string(),
            height: 1_000,
            time: Timestamp::from_nanos(4_000_000 + 1),
        });
        // Collector_1 buys 1 NFT from Minter_1 during round 4
        // Price is 200_000 uflix
        // Mint flix to collector_1
        mint_to_address(&mut app, collector_1.to_string(), coins(200000, "uflix"));
        let _res = app
            .execute_contract(
                collector_1.clone(),
                Addr::unchecked(minter_1_addr.clone()),
                &MinterExecuteMsg::Mint {},
                &[coin(200000, "uflix")],
            )
            .unwrap();

        // Query Onft
        let collection_1 = query_onft_collection(app.storage(), minter_1_addr.clone());
        assert_eq!(collection_1.onfts.len(), 4);

        // Collector_1 waits for public mint to start and buys 1 NFT from Minter_1
        // Price is 5_000_000 uflix
        app.set_block(BlockInfo {
            chain_id: "test_1".to_string(),
            height: 1_000,
            time: Timestamp::from_nanos(1_000_000_000 + 1),
        });
        // Mint flix to
        mint_to_address(&mut app, collector_1.to_string(), coins(5000000, "uflix"));

        // Creator buys 1 NFT from Minter_1
        let res = app
            .execute_contract(
                creator.clone(),
                Addr::unchecked(minter_1_addr.clone()),
                &MinterExecuteMsg::Mint {},
                &[coin(5000000, "uflix")],
            )
            .unwrap();
    }
}
