#![cfg(test)]
use cosmwasm_std::{
    coin, coins, to_json_binary, Addr, BlockInfo, Empty, QueryRequest, Timestamp, Uint128,
    WasmQuery,
};
use cw_multi_test::{BankSudo, Executor, SudoMsg};
use minter_types::UserDetails;
use minter_types::{QueryMsg as MinterQueryMsg, Token};

use omniflix_minter::msg::{ExecuteMsg as MinterExecuteMsg, MinterExtensionQueryMsg};
use omniflix_minter_factory::msg::ExecuteMsg as FactoryExecuteMsg;

use crate::helpers::utils::{
    get_contract_address_from_res, return_factory_inst_message, return_minter_instantiate_msg,
    return_rounds,
};

use crate::{helpers::setup::setup, helpers::utils::query_onft_collection};
use omniflix_minter::error::ContractError as MinterContractError;

use omniflix_round_whitelist::error::ContractError as RoundWhitelistContractError;

#[test]
pub fn test_mint() {
    let (
        mut app,
        test_addresses,
        minter_factory_code_id,
        minter_code_id,
        _round_whitelist_factory_code_id,
        _round_whitelist_code_id,
        _open_edition_minter_code_id,
        _open_edition_minter_factory_code_id,
    ) = setup();
    let admin = test_addresses.admin;
    let creator = test_addresses.creator;
    let collector = test_addresses.collector;

    let factory_inst_msg = return_factory_inst_message(minter_code_id);
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

    // minting before start time
    let error = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1000000, "uflix")],
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
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(1_000_000_000 + 1),
    });

    // mint with incorrect denom
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

    // mint with incorrect amount
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
    //minting after end time
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(1_000_000_000 + 1_000_000_000 + 1),
    });
    let error = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::PublicMintingEnded {});

    // Reset to a valid time
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(1_000_000_000 + 1),
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
            &[coin(1000000, "uflix")],
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
        collection.onfts.into_iter().next().unwrap().owner,
        collector
    );
    // Now query contract
    let token_data: UserDetails = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: minter_address.clone(),
            msg: to_json_binary::<MinterQueryMsg<MinterExtensionQueryMsg>>(
                &MinterQueryMsg::MintedTokens {
                    address: collector.clone().into_string(),
                },
            )
            .unwrap(),
        }))
        .unwrap();
    assert_eq!(token_data.minted_tokens[0].token_id, token_id);

    // Check total tokens remaining
    let total_tokens_remaining_data: Vec<Token> = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: minter_address.clone(),
            msg: to_json_binary(&MinterQueryMsg::Extension(
                omniflix_minter::msg::MinterExtensionQueryMsg::MintableTokens {},
            ))
            .unwrap(),
        }))
        .unwrap();
    assert_eq!(total_tokens_remaining_data.len(), 999);
    assert!(!total_tokens_remaining_data
        .iter()
        .any(|x| x.token_id == token_id));
    // Try minting second time with same address
    let error = app
        .execute_contract(
            Addr::unchecked(collector.clone()),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::AddressReachedMintLimit {});

    // Create a loop from 1 to 999 and mint every remaining token to receivers
    for i in 1..=999 {
        // add i as string to the end of the collector address
        let collector = Addr::unchecked(format!("{}{}", collector, i));
        // Mint tokens to mint nft
        app.sudo(SudoMsg::Bank(BankSudo::Mint {
            to_address: collector.to_string(),
            amount: coins(1000000, "uflix"),
        }))
        .unwrap();
        // Mint
        let _res = app
            .execute_contract(
                collector.clone(),
                Addr::unchecked(minter_address.clone()),
                &MinterExecuteMsg::Mint {},
                &[coin(1000000, "uflix")],
            )
            .unwrap();
    }
    // query total mintable tokens
    let mintable_tokens_data: Vec<Token> = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: minter_address.clone(),
            msg: to_json_binary(&MinterQueryMsg::Extension(
                omniflix_minter::msg::MinterExtensionQueryMsg::MintableTokens {},
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
                omniflix_minter::msg::MinterExtensionQueryMsg::TotalTokensRemaining {},
            ))
            .unwrap(),
        }))
        .unwrap();
    assert_eq!(total_tokens_remaining_data, 0);

    // Check minted tokens for address we will unwrap every query so if not failed in loop we minted correctly
    // Every token should be diffirent
    let mut minted_list: Vec<Token> = Vec::new();

    for i in 1..=999 {
        let user_details_data: UserDetails = app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: minter_address.clone(),
                msg: to_json_binary::<MinterQueryMsg<MinterExtensionQueryMsg>>(
                    &MinterQueryMsg::MintedTokens {
                        address: Addr::unchecked(format!("{}{}", collector, i)).to_string(),
                    },
                )
                .unwrap(),
            }))
            .unwrap();
        minted_list.push(user_details_data.minted_tokens[0].clone());
    }
    assert_eq!(minted_list.len(), 999);
    minted_list.sort_by(|a, b| a.token_id.cmp(&b.token_id));
    for i in 0..=997 {
        assert_ne!(minted_list[i], minted_list[i + 1]);
    }
    // Now there is no tokens left to mint
    // Try minting with collector1001
    app.sudo(SudoMsg::Bank(BankSudo::Mint {
        to_address: Addr::unchecked("collector1001".to_string()).to_string(),
        amount: coins(1000000, "uflix"),
    }))
    .unwrap();

    let error = app
        .execute_contract(
            Addr::unchecked("collector1001".to_string()),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let res = error;
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::NoTokensLeftToMint {});
}

#[test]
pub fn test_mint_admin() {
    let (
        mut app,
        test_addresses,
        minter_factory_code_id,
        minter_code_id,
        _round_whitelist_factory_code_id,
        _round_whitelist_code_id,
        _open_edition_minter_code_id,
        _open_edition_minter_factory_code_id,
    ) = setup();
    let admin = test_addresses.admin;
    let creator = test_addresses.creator;
    let collector = test_addresses.collector;

    let factory_inst_msg = return_factory_inst_message(minter_code_id);
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
                token_id: Some("334".to_string()),
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
                token_id: Some("334".to_string()),
            },
            &[],
        )
        .unwrap();
    let token_id: String = res.events[1].attributes[2].value.clone();
    let collection_id: String = res.events[1].attributes[3].value.clone();
    assert_eq!(collection_id, "id".to_string());
    assert_eq!(token_id, "334".to_string());

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
            msg: to_json_binary(&MinterQueryMsg::<Empty>::MintedTokens {
                address: "gift_recipient".to_string(),
            })
            .unwrap(),
        }))
        .unwrap();
    assert_eq!(token_data.minted_tokens[0].token_id, token_id);

    // Check total tokens remaining
    let total_tokens_remaining_data: Vec<Token> = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: minter_address.clone(),
            msg: to_json_binary(&MinterQueryMsg::Extension(
                MinterExtensionQueryMsg::MintableTokens {},
            ))
            .unwrap(),
        }))
        .unwrap();
    assert_eq!(total_tokens_remaining_data.len(), 999);
    assert!(!total_tokens_remaining_data
        .iter()
        .any(|x| x.token_id == token_id));

    // Try minting with same Id
    let error = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::MintAdmin {
                recipient: "gift_recipient".to_string(),
                token_id: Some("334".to_string()),
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
            msg: to_json_binary(&MinterQueryMsg::<Empty>::MintedTokens {
                address: "gift_recipient".to_string(),
            })
            .unwrap(),
        }))
        .unwrap();
    assert_eq!(token_data.minted_tokens[1].token_id, token_id);
}

#[test]
pub fn test_mint_with_whitelist() {
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
    let collector = test_addresses.collector;

    let factory_inst_msg = return_factory_inst_message(minter_code_id);
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

    // Right now default configuration private minting starts at 2_000 and ends at 5_000
    // Public mint starts at 5_000 and ends at 1_000_000_000
    // Now is 1_000 as default

    // Try instantiating minter with already active whitelist
    let round_whitelist_inst_msg = whitelist_types::InstantiateMsg {
        admin: admin.to_string(),
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
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(2_000 + 1),
    });
    let round_whitelist_address = get_contract_address_from_res(res.clone());

    let mut minter_inst_msg = return_minter_instantiate_msg();
    minter_inst_msg.init.whitelist_address = Some(round_whitelist_address.clone());
    minter_inst_msg.init.per_address_limit = Some(2);

    let create_minter_msg = FactoryExecuteMsg::CreateMinter {
        msg: minter_inst_msg,
    };

    let error = app
        .execute_contract(
            admin.clone(),
            minter_factory_addr.clone(),
            &create_minter_msg.clone(),
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let res = error.source().unwrap().source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::WhitelistAlreadyActive {});

    // Reset block time
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(1_000),
    });

    let res = app
        .execute_contract(
            admin.clone(),
            minter_factory_addr,
            &create_minter_msg.clone(),
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let minter_address = get_contract_address_from_res(res.clone());
    // Try minting when whitelist is not active
    let error = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::WhitelistNotActive {});

    // Try minting with non whitelisted address
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(2_000 + 1),
    });

    let error = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::AddressNotWhitelisted {});

    // Try minting round one with wrong denom
    let error = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(
        error,
        &MinterContractError::PaymentError(cw_utils::PaymentError::ExtraDenom("uflix".to_string()))
    );
    // Try minting round one with wrong amount
    let error = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(100000 + 1, "diffirent_denom")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(
        error,
        &MinterContractError::IncorrectPaymentAmount {
            expected: Uint128::from(1000000u128),
            sent: Uint128::from(100001u128)
        }
    );

    // Try minting round one with correct denom and amount
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1000000, "diffirent_denom")],
        )
        .unwrap();
    let token_id: String = res.events[1].attributes[2].value.clone();
    let collection_id: String = res.events[1].attributes[3].value.clone();
    // We are quering collection to check if it is minted from our mocked onft keeper
    let collection = query_onft_collection(app.storage(), minter_address.clone());
    assert_eq!(collection.denom.clone().unwrap().id, collection_id);
    assert_eq!(
        collection.onfts.clone().into_iter().next().unwrap().id,
        token_id
    );

    // At first round only one address can mint and per address limit is 1
    // Try minting once again with same address
    let error = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1000000, "diffirent_denom")],
        )
        .unwrap_err();
    let res = error;
    let error = res.downcast_ref::<RoundWhitelistContractError>().unwrap();
    assert_eq!(
        error,
        &RoundWhitelistContractError::RoundReachedMintLimit {}
    );

    // Set block between round 1 and 2
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(3_000 + 1),
    });

    // Try minting with collector address
    let error = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1000000, "diffirent_denom")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::WhitelistNotActive {});

    // Set block between to round 2
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(4_000 + 1),
    });

    // Try minting with collector address
    let error = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1000000, "diffirent_denom")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::AddressNotWhitelisted {});

    // Try minting with creator address
    // Second round mint price is 2_000_000 uflix
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1000000, "uflix")],
        )
        .unwrap();
    let token_id: String = res.events[1].attributes[2].value.clone();
    let _collection_id: String = res.events[1].attributes[3].value.clone();
    // We are quering collection to check if it is minted from our mocked onft keeper
    let collection = query_onft_collection(app.storage(), minter_address.clone());
    assert_eq!(collection.onfts.clone()[1].id, token_id);
}
