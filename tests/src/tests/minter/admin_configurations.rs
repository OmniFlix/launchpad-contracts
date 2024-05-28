use std::str::FromStr;

use cosmwasm_std::{coin, BlockInfo, Decimal, Timestamp};
use cosmwasm_std::{Addr, StdError};
use cw_multi_test::Executor;

use minter_types::config::Config;
use minter_types::token_details::{Token, TokenDetails, TokenDetailsError};
use minter_types::types::AuthDetails;

use minter_types::msg::QueryMsg;

use omniflix_minter_factory::msg::ExecuteMsg as FactoryExecuteMsg;
use whitelist_types::CreateWhitelistMsg;

use crate::helpers::mock_messages::factory_mock_messages::{
    return_minter_factory_inst_message, return_round_whitelist_factory_inst_message,
};
use crate::helpers::mock_messages::minter_mock_messages::return_minter_instantiate_msg;
use crate::helpers::mock_messages::whitelist_mock_messages::return_round_configs;
use crate::helpers::utils::get_contract_address_from_res;

use crate::helpers::setup::setup;
use omniflix_minter::error::ContractError as MinterContractError;
use omniflix_minter::msg::ExecuteMsg as MinterExecuteMsg;
use omniflix_minter::msg::MinterExtensionQueryMsg;
type MinterQueryMsg = QueryMsg<MinterExtensionQueryMsg>;

#[test]
fn update_whitelist() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let collector = res.test_accounts.collector;
    let minter_factory_code_id = res.minter_factory_code_id;
    let minter_code_id = res.minter_code_id;
    let round_whitelist_factory_code_id = res.round_whitelist_factory_code_id;
    let round_whitelist_code_id = res.round_whitelist_code_id;
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

    // Now is 1_000 as default
    let round_whitelist_inst_msg = CreateWhitelistMsg {
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
            round_whitelist_factory_addr.clone(),
            &create_round_whitelist_msg,
            &[coin(1000000, "uflix")],
        )
        .unwrap();

    let round_whitelist_address = get_contract_address_from_res(res.clone());

    let mut minter_inst_msg = return_minter_instantiate_msg();
    // Unwrap init
    let mut init = minter_inst_msg.init.unwrap().clone();
    init.whitelist_address = Some(round_whitelist_address.clone());
    minter_inst_msg.init = Some(init);

    let create_minter_msg = FactoryExecuteMsg::CreateMinter {
        msg: minter_inst_msg,
    };

    let res = app
        .execute_contract(
            admin.clone(),
            factory_addr,
            &create_minter_msg.clone(),
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let minter_address = get_contract_address_from_res(res.clone());
    // Can not set whitelist if current round is active
    app.set_block(BlockInfo {
        time: rounds.clone()[0].round.start_time,
        height: 1,
        chain_id: "cosmos".to_string(),
    });
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::UpdateWhitelistAddress {
                address: round_whitelist_address.clone(),
            },
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::WhitelistAlreadyActive {});

    // Reset time
    app.set_block(BlockInfo {
        time: Timestamp::from_nanos(1_000),
        height: 1,
        chain_id: "cosmos".to_string(),
    });

    // Can not set whitelist if provided address is not a whitelist
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::UpdateWhitelistAddress {
                address: minter_address.clone(),
            },
            &[],
        )
        .unwrap_err();
    assert!(res
        .source()
        .unwrap()
        .downcast_ref::<MinterContractError>()
        .is_some());
    // Can not set whitelist if provided address is a already active whitelist
    let mut rounds = return_round_configs();
    rounds[0].round.start_time = Timestamp::from_nanos(1_000);
    let round_whitelist_inst_msg = CreateWhitelistMsg {
        admin: admin.to_string(),
        rounds: rounds.clone(),
    };
    let create_round_whitelist_msg =
        omniflix_round_whitelist_factory::msg::ExecuteMsg::CreateWhitelist {
            msg: round_whitelist_inst_msg,
        };
    // Create a whitelist
    let res = app
        .execute_contract(
            admin.clone(),
            round_whitelist_factory_addr.clone(),
            &create_round_whitelist_msg,
            &[coin(1000000, "uflix")],
        )
        .unwrap();
    let whitelist_address = get_contract_address_from_res(res);

    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::UpdateWhitelistAddress {
                address: whitelist_address.clone(),
            },
            &[],
        )
        .unwrap_err();

    let error = res.source().unwrap();
    let error = error.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::WhitelistAlreadyActive {});

    // Create a non active whitelist
    let mut rounds = return_round_configs();
    rounds[0].round.start_time = Timestamp::from_nanos(2_000);
    let round_whitelist_inst_msg = CreateWhitelistMsg {
        admin: admin.to_string(),
        rounds: rounds.clone(),
    };
    let create_round_whitelist_msg =
        omniflix_round_whitelist_factory::msg::ExecuteMsg::CreateWhitelist {
            msg: round_whitelist_inst_msg,
        };
    // Create a whitelist
    let res = app
        .execute_contract(
            admin.clone(),
            round_whitelist_factory_addr.clone(),
            &create_round_whitelist_msg,
            &[coin(1000000, "uflix")],
        )
        .unwrap();
    let whitelist_address = get_contract_address_from_res(res);

    // Non admin can not set whitelist
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::UpdateWhitelistAddress {
                address: whitelist_address.clone(),
            },
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::Unauthorized {});

    // Set whitelist
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::UpdateWhitelistAddress {
                address: whitelist_address.clone(),
            },
            &[],
        )
        .unwrap();
    let config: Config = app
        .wrap()
        .query_wasm_smart(minter_address.clone(), &MinterQueryMsg::Config {})
        .unwrap();
    assert_eq!(
        config.whitelist_address,
        Some(Addr::unchecked(whitelist_address.clone()))
    );
}
#[test]
fn burn_remaining_tokens() {
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

    let res = app
        .execute_contract(
            admin.clone(),
            factory_addr,
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let minter_address = get_contract_address_from_res(res.clone());

    // Query mintable tokens
    let mintable_tokens: Vec<(u32, Token)> = app
        .wrap()
        .query_wasm_smart(
            minter_address.clone(),
            &MinterQueryMsg::Extension(MinterExtensionQueryMsg::MintableTokens {
                start_after: None,
                limit: None,
            }),
        )
        .unwrap();
    assert_eq!(mintable_tokens.len(), 50);

    // Query tokens remaining
    let tokens_remaining: u32 = app
        .wrap()
        .query_wasm_smart(
            minter_address.clone(),
            &MinterQueryMsg::Extension(MinterExtensionQueryMsg::TotalTokensRemaining {}),
        )
        .unwrap();
    assert_eq!(tokens_remaining, 50);

    // Burn remaining tokens execution sets mintable tokens to 0
    // Try minting before
    app.set_block(BlockInfo {
        time: public_start_time,
        height: 1,
        chain_id: "cosmos".to_string(),
    });
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[public_minting_price.clone()],
        )
        .unwrap();

    // Query mintable tokens
    let mintable_tokens: Vec<(u32, Token)> = app
        .wrap()
        .query_wasm_smart(
            minter_address.clone(),
            &MinterQueryMsg::Extension(MinterExtensionQueryMsg::MintableTokens {
                start_after: None,
                limit: None,
            }),
        )
        .unwrap();
    assert_eq!(mintable_tokens.len(), 49);

    // Query tokens remaining
    let tokens_remaining: u32 = app
        .wrap()
        .query_wasm_smart(
            minter_address.clone(),
            &MinterQueryMsg::Extension(MinterExtensionQueryMsg::TotalTokensRemaining {}),
        )
        .unwrap();
    assert_eq!(tokens_remaining, 49);

    // Non admin can not burn remaining tokens
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::BurnRemainingTokens {},
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::Unauthorized {});

    // Admin can burn remaining tokens
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::BurnRemainingTokens {},
            &[],
        )
        .unwrap();

    // Query mintable tokens
    let mintable_tokens: Vec<(u32, Token)> = app
        .wrap()
        .query_wasm_smart(
            minter_address.clone(),
            &MinterQueryMsg::Extension(MinterExtensionQueryMsg::MintableTokens {
                start_after: None,
                limit: None,
            }),
        )
        .unwrap();
    assert_eq!(mintable_tokens.len(), 0);

    // Query tokens remaining
    let tokens_remaining: u32 = app
        .wrap()
        .query_wasm_smart(
            minter_address.clone(),
            &MinterQueryMsg::Extension(MinterExtensionQueryMsg::TotalTokensRemaining {}),
        )
        .unwrap();
    assert_eq!(tokens_remaining, 0);

    // Try minting after
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[public_minting_price.clone()],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::NoTokensLeftToMint {});
}

#[test]
fn update_royalty_ratio() {
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

    let res = app
        .execute_contract(
            admin.clone(),
            factory_addr,
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let minter_address = get_contract_address_from_res(res.clone());

    // Non admin can not update royalty ratio
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::UpdateRoyaltyRatio {
                ratio: "0.2".to_string(),
            },
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::Unauthorized {});

    // Invalid ratio
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::UpdateRoyaltyRatio {
                ratio: "One".to_string(),
            },
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(
        error,
        &MinterContractError::Std(StdError::generic_err("Error parsing whole"))
    );

    // Send ratio more than 1
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::UpdateRoyaltyRatio {
                ratio: "1.1".to_string(),
            },
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(
        error,
        &MinterContractError::TokenDetailsError(TokenDetailsError::InvalidRoyaltyRatio {})
    );

    // Update royalty ratio
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::UpdateRoyaltyRatio {
                ratio: "0.2".to_string(),
            },
            &[],
        )
        .unwrap();

    let token_details: TokenDetails = app
        .wrap()
        .query_wasm_smart(minter_address.clone(), &MinterQueryMsg::TokenDetails {})
        .unwrap();
    assert_eq!(
        token_details.royalty_ratio,
        Decimal::from_str("0.2").unwrap()
    );
}

#[test]
fn update_mint_price() {
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

    let res = app
        .execute_contract(
            admin.clone(),
            factory_addr,
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let minter_address = get_contract_address_from_res(res.clone());

    // Non admin can not update mint price
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::UpdateMintPrice {
                mint_price: coin(2_000_000, "uflix"),
            },
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::Unauthorized {});

    // Update mint price
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::UpdateMintPrice {
                mint_price: coin(2_000_000, "uflix"),
            },
            &[],
        )
        .unwrap();

    let config: Config = app
        .wrap()
        .query_wasm_smart(minter_address.clone(), &MinterQueryMsg::Config {})
        .unwrap();
    assert_eq!(config.mint_price, coin(2_000_000, "uflix"));
}

#[test]
fn randomize_list() {
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

    let res = app
        .execute_contract(
            admin.clone(),
            factory_addr,
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let minter_address = get_contract_address_from_res(res.clone());

    // Query mintable tokens
    let mintable_tokens: Vec<(u32, Token)> = app
        .wrap()
        .query_wasm_smart(
            minter_address.clone(),
            &MinterQueryMsg::Extension(MinterExtensionQueryMsg::MintableTokens {
                start_after: None,
                limit: None,
            }),
        )
        .unwrap();
    let first_token = mintable_tokens[0].clone();

    // Non admin can not randomize list
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::RandomizeList {},
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::Unauthorized {});

    // Randomize list
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::RandomizeList {},
            &[],
        )
        .unwrap();

    // Query mintable tokens
    let mintable_tokens: Vec<(u32, Token)> = app
        .wrap()
        .query_wasm_smart(
            minter_address.clone(),
            &MinterQueryMsg::Extension(MinterExtensionQueryMsg::MintableTokens {
                start_after: None,
                limit: None,
            }),
        )
        .unwrap();
    let first_token_after_randomize = mintable_tokens[0].clone();
    assert_ne!(first_token, first_token_after_randomize);
}

#[test]
fn update_admin() {
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

    let res = app
        .execute_contract(
            admin.clone(),
            factory_addr,
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let minter_address = get_contract_address_from_res(res.clone());

    // Non admin can not update admin
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::UpdateAdmin {
                admin: collector.clone().into_string(),
            },
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::Unauthorized {});

    // Update admin
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::UpdateAdmin {
                admin: collector.clone().into_string(),
            },
            &[],
        )
        .unwrap();

    let auth_details: AuthDetails = app
        .wrap()
        .query_wasm_smart(minter_address.clone(), &MinterQueryMsg::AuthDetails {})
        .unwrap();
    assert_eq!(auth_details.admin, collector.clone());
}

#[test]
fn update_payment_collector() {
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

    let res = app
        .execute_contract(
            admin.clone(),
            factory_addr,
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let minter_address = get_contract_address_from_res(res.clone());

    // Non admin can not update payment collector
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::UpdatePaymentCollector {
                payment_collector: collector.clone().into_string(),
            },
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::Unauthorized {});

    // Update payment collector
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::UpdatePaymentCollector {
                payment_collector: collector.clone().into_string(),
            },
            &[],
        )
        .unwrap();

    let auth_details: AuthDetails = app
        .wrap()
        .query_wasm_smart(minter_address.clone(), &MinterQueryMsg::AuthDetails {})
        .unwrap();
    assert_eq!(auth_details.payment_collector, collector.clone());
}
