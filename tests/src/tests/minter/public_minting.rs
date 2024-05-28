use cosmwasm_std::Addr;
use cosmwasm_std::{coin, to_json_binary, BlockInfo, QueryRequest, Timestamp, Uint128, WasmQuery};
use cw_multi_test::Executor;

use minter_types::token_details::Token;
use minter_types::types::UserDetails;

use minter_types::msg::QueryMsg;

use omniflix_minter_factory::msg::ExecuteMsg as FactoryExecuteMsg;

use crate::helpers::mock_messages::factory_mock_messages::return_minter_factory_inst_message;
use crate::helpers::mock_messages::minter_mock_messages::return_minter_instantiate_msg;
use crate::helpers::utils::{get_contract_address_from_res, mint_to_address};

use crate::{helpers::setup::setup, helpers::utils::query_onft_collection};
use omniflix_minter::error::ContractError as MinterContractError;
use omniflix_minter::msg::ExecuteMsg as MinterExecuteMsg;
use omniflix_minter::msg::MinterExtensionQueryMsg;

type MinterQueryMsg = QueryMsg<MinterExtensionQueryMsg>;

#[test]
fn minter_public_minting() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let collector = res.test_accounts.collector;
    let minter_factory_code_id = res.minter_factory_code_id;
    let minter_code_id = res.minter_code_id;
    let mut app = res.app;

    let factory_inst_msg = return_minter_factory_inst_message(minter_code_id);
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
        msg: minter_inst_msg.clone(),
    };
    let public_minting_price = minter_inst_msg.init.clone().unwrap().mint_price;
    let public_start_time = minter_inst_msg.init.clone().unwrap().start_time;
    let public_end_time = minter_inst_msg.init.clone().unwrap().end_time.unwrap();

    let res = app
        .execute_contract(
            admin.clone(),
            factory_addr,
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let minter_address = get_contract_address_from_res(res.clone());
    // First check queries
    // Query mintable tokens
    let mintable_tokens_data: Vec<(u32, Token)> = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: minter_address.clone(),
            msg: to_json_binary(&MinterQueryMsg::Extension(
                MinterExtensionQueryMsg::MintableTokens {
                    start_after: None,
                    limit: None,
                },
            ))
            .unwrap(),
        }))
        .unwrap();
    assert_eq!(mintable_tokens_data.len(), 50);

    // Query total tokens remaining
    let total_tokens_remaining_data: u32 = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: minter_address.clone(),
            msg: to_json_binary(&MinterQueryMsg::Extension(
                MinterExtensionQueryMsg::TotalTokensRemaining {},
            ))
            .unwrap(),
        }))
        .unwrap();
    assert_eq!(total_tokens_remaining_data, 50);

    // Query config
    let total_minted_count: u32 = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: minter_address.clone(),
            msg: to_json_binary(&MinterQueryMsg::TotalMintedCount {}).unwrap(),
        }))
        .unwrap();
    assert_eq!(total_minted_count, 0);

    // minting before start time
    let error = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[public_minting_price.clone()],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(
        error,
        &MinterContractError::MintingNotStarted {
            start_time: Timestamp::from_nanos(1_000_000_000),
            current_time: Timestamp::from_nanos(1_000)
        }
    );

    // Set block time to start time
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: public_start_time,
    });

    // Mint with incorrect denom
    let error = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1000000, "incorrect_denom")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(
        error,
        &MinterContractError::PaymentError(cw_utils::PaymentError::ExtraDenom(
            "incorrect_denom".to_string()
        ))
    );

    // Mint with incorrect amount
    let error = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(100000, "uflix")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(
        error,
        &MinterContractError::IncorrectPaymentAmount {
            expected: Uint128::from(1000000u128),
            sent: Uint128::from(100000u128)
        }
    );

    // Set block time to after end time
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(public_end_time.nanos() + 1),
    });

    // Minting after end time
    let error = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[public_minting_price.clone()],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::PublicMintingEnded {});

    // Reset to a valid time
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: public_start_time,
    });

    // Query uflix balance of creator before mint
    let creator_balance_before_mint: Uint128 = app
        .wrap()
        .query_balance(creator.to_string(), "uflix".to_string())
        .unwrap()
        .amount;
    // Mint
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[public_minting_price.clone()],
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
        creator_balance_before_mint + Uint128::new(public_minting_price.amount.u128())
    );

    let token_id: String = res.events[1].attributes[2].value.clone();
    let collection_id: String = res.events[1].attributes[3].value.clone();
    // We are quering collection to check if it is minted from our mocked onft keeper
    let collection = query_onft_collection(app.storage(), minter_address.clone());
    assert_eq!(collection.denom.clone().unwrap().id, collection_id);
    assert_eq!(
        collection.onfts.clone().into_iter().next().unwrap().id,
        token_id
    );
    assert_eq!(
        collection.onfts.clone().into_iter().next().unwrap().owner,
        collector
    );
    // Now query contract
    let user_minting_details: UserDetails = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: minter_address.clone(),
            msg: to_json_binary::<MinterQueryMsg>(&MinterQueryMsg::UserMintingDetails {
                address: collector.clone().into_string(),
            })
            .unwrap(),
        }))
        .unwrap();
    assert_eq!(user_minting_details.minted_tokens[0].token_id, token_id);
    assert_eq!(user_minting_details.total_minted_count, 1);
    assert_eq!(user_minting_details.public_mint_count, 1);

    // Check total tokens remaining
    let total_tokens_remaining_data: u32 = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: minter_address.clone(),
            msg: to_json_binary(&MinterQueryMsg::Extension(
                MinterExtensionQueryMsg::TotalTokensRemaining {},
            ))
            .unwrap(),
        }))
        .unwrap();
    assert_eq!(total_tokens_remaining_data, 49);

    // Try minting second time with same address
    let error = app
        .execute_contract(
            Addr::unchecked(collector.clone()),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[public_minting_price.clone()],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::AddressReachedMintLimit {});

    // Create a loop from 1 to 999 and mint every remaining token to receivers
    for i in 1..=49 {
        // add i as string to the end of the collector address
        let collector = Addr::unchecked(format!("{}{}", collector, i));
        // Mint tokens to mint nft
        mint_to_address(
            &mut app,
            collector.clone().into_string(),
            vec![public_minting_price.clone()],
        );
        // Mint
        let _res = app
            .execute_contract(
                collector.clone(),
                Addr::unchecked(minter_address.clone()),
                &MinterExecuteMsg::Mint {},
                &[public_minting_price.clone()],
            )
            .unwrap();
    }
    // query total mintable tokens
    let mintable_tokens_data: Vec<Token> = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: minter_address.clone(),
            msg: to_json_binary(&MinterQueryMsg::Extension(
                MinterExtensionQueryMsg::MintableTokens {
                    start_after: None,
                    limit: None,
                },
            ))
            .unwrap(),
        }))
        .unwrap();
    assert_eq!(mintable_tokens_data.len(), 0);

    // query total tokens remaining
    let total_tokens_remaining_data: u32 = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: minter_address.clone(),
            msg: to_json_binary(&MinterQueryMsg::Extension(
                MinterExtensionQueryMsg::TotalTokensRemaining {},
            ))
            .unwrap(),
        }))
        .unwrap();
    assert_eq!(total_tokens_remaining_data, 0);

    // Check minted tokens for address we will unwrap every query so if not failed in loop we minted correctly
    // Every token should be diffirent
    let mut minted_list: Vec<Token> = Vec::new();

    for i in 1..=49 {
        let user_details_data: UserDetails = app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: minter_address.clone(),
                msg: to_json_binary::<MinterQueryMsg>(&MinterQueryMsg::UserMintingDetails {
                    address: Addr::unchecked(format!("{}{}", collector, i)).to_string(),
                })
                .unwrap(),
            }))
            .unwrap();
        minted_list.push(user_details_data.minted_tokens[0].clone());
    }
    assert_eq!(minted_list.len(), 49);
    minted_list.sort_by(|a, b| a.token_id.cmp(&b.token_id));
    for i in 0..=47 {
        assert_ne!(minted_list[i], minted_list[i + 1]);
    }
    // Now there is no tokens left to mint
    // Try minting with collector1001
    mint_to_address(
        &mut app,
        "collector1001".to_string(),
        vec![public_minting_price.clone()],
    );

    let error = app
        .execute_contract(
            Addr::unchecked("collector1001".to_string()),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[public_minting_price.clone()],
        )
        .unwrap_err();
    let res = error;
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::NoTokensLeftToMint {});
}

#[test]
pub fn mint_admin() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let collector = res.test_accounts.collector;
    let minter_factory_code_id = res.minter_factory_code_id;
    let minter_code_id = res.minter_code_id;
    let mut app = res.app;

    let factory_inst_msg = return_minter_factory_inst_message(minter_code_id);
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
    let res = app
        .execute_contract(
            admin.clone(),
            factory_addr,
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let minter_address = get_contract_address_from_res(res.clone());

    // Try minting with money but non payable for admin
    let error = app
        .execute_contract(
            admin.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::MintAdmin {
                recipient: "gift_recipient".to_string(),
                token_id: Some("334".to_string()),
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(
        error,
        &MinterContractError::PaymentError(cw_utils::PaymentError::NonPayable {})
    );

    // Try with non admin
    let error = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::MintAdmin {
                recipient: "gift_recipient".to_string(),
                token_id: Some("34".to_string()),
            },
            &[],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::Unauthorized {});

    // Try minting creator does not need to wait for start time
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(1_000_000_000 - 1),
    });
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::MintAdmin {
                recipient: "gift_recipient".to_string(),
                token_id: Some("34".to_string()),
            },
            &[],
        )
        .unwrap();
    let token_id: String = res.events[1].attributes[2].value.clone();
    let collection_id: String = res.events[1].attributes[3].value.clone();
    assert_eq!(collection_id, "id".to_string());
    assert_eq!(token_id, "34".to_string());

    // Query onft collection
    let collection = query_onft_collection(app.storage(), minter_address.clone());
    assert_eq!(collection.denom.clone().unwrap().id, collection_id);
    assert_eq!(
        collection.onfts.clone().into_iter().next().unwrap().id,
        token_id
    );
    assert_eq!(
        collection.onfts.clone().into_iter().next().unwrap().owner,
        "gift_recipient".to_string()
    );
    assert_eq!(
        collection
            .onfts
            .clone()
            .into_iter()
            .next()
            .unwrap()
            .royalty_share,
        "100000000000000000".to_string()
    );

    // Query minted tokens
    let token_data: UserDetails = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: minter_address.clone(),
            msg: to_json_binary(&MinterQueryMsg::UserMintingDetails {
                address: "gift_recipient".to_string(),
            })
            .unwrap(),
        }))
        .unwrap();
    assert_eq!(token_data.minted_tokens[0].token_id, token_id);

    // Check total tokens remaining
    let total_tokens_remaining_data: Vec<(u32, Token)> = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: minter_address.clone(),
            msg: to_json_binary(&MinterQueryMsg::Extension(
                MinterExtensionQueryMsg::MintableTokens {
                    start_after: None,
                    limit: None,
                },
            ))
            .unwrap(),
        }))
        .unwrap();
    assert_eq!(total_tokens_remaining_data.len(), 49);
    assert!(!total_tokens_remaining_data
        .iter()
        .any(|x| x.1.token_id == token_id));

    // Try minting with same Id
    let error = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::MintAdmin {
                recipient: "gift_recipient".to_string(),
                token_id: Some("34".to_string()),
            },
            &[],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::TokenIdNotMintable {});

    // Try minting with without denom id
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::MintAdmin {
                recipient: "gift_recipient".to_string(),
                token_id: None,
            },
            &[],
        )
        .unwrap();
    let token_id: String = res.events[1].attributes[2].value.clone();

    // Check minted tokens for address
    let token_data: UserDetails = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: minter_address.clone(),
            msg: to_json_binary(&MinterQueryMsg::UserMintingDetails {
                address: "gift_recipient".to_string(),
            })
            .unwrap(),
        }))
        .unwrap();
    assert_eq!(token_data.minted_tokens[1].token_id, token_id);
}
