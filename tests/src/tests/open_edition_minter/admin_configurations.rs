#![cfg(test)]
use cosmwasm_std::{coin, Addr, BlockInfo, Coin, StdError, Timestamp};

use cw_multi_test::Executor;
use minter_types::types::AuthDetails;

use minter_types::config::Config;
use minter_types::token_details::{TokenDetails, TokenDetailsError};

use omniflix_open_edition_minter_factory::msg::ExecuteMsg as OpenEditionMinterFactoryExecuteMsg;
use whitelist_types::CreateWhitelistMsg;

use crate::helpers::mock_messages::whitelist_mock_messages::return_rounds;
use crate::helpers::utils::get_contract_address_from_res;

use crate::helpers::mock_messages::factory_mock_messages::{
    return_open_edition_minter_factory_inst_message, return_round_whitelist_factory_inst_message,
};

use crate::helpers::mock_messages::oem_mock_messages::return_open_edition_minter_inst_msg;

use crate::helpers::utils::query_onft_collection;

use crate::helpers::setup::setup;
use omniflix_open_edition_minter::msg::OEMQueryExtension;

use minter_types::msg::QueryMsg as BaseMinterQueryMsg;

use omniflix_open_edition_minter::msg::ExecuteMsg as OpenEditionMinterExecuteMsg;

use omniflix_open_edition_minter::error::ContractError as OpenEditionMinterError;

type OpenEditionMinterQueryMsg = BaseMinterQueryMsg<OEMQueryExtension>;

#[test]
fn update_whitelist() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let collector = res.test_accounts.collector;
    let open_edition_minter_factory_code_id = res.open_edition_minter_factory_code_id;
    let open_edition_minter_code_id = res.open_edition_minter_code_id;
    let round_whitelist_factory_code_id = res.round_whitelist_factory_code_id;
    let round_whitelist_code_id = res.round_whitelist_code_id;
    let mut app = res.app;

    // Instantiate the oem minter factory
    let open_edition_minter_factory_instantiate_msg =
        return_open_edition_minter_factory_inst_message(open_edition_minter_code_id, None);

    let open_edition_minter_factory_address = app
        .instantiate_contract(
            open_edition_minter_factory_code_id,
            admin.clone(),
            &open_edition_minter_factory_instantiate_msg.clone(),
            &[],
            "Open Edition Minter Factory",
            None,
        )
        .unwrap();

    // Create a whitelist factory
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

    let rounds = return_rounds();

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

    // Create a minter
    let mut open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
    open_edition_minter_instantiate_msg.init.whitelist_address = Some(whitelist_address.clone());
    let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateOpenEditionMinter {
        msg: open_edition_minter_instantiate_msg.clone(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[Coin::new(2000000, "uflix")],
        )
        .unwrap();
    let minter_address = get_contract_address_from_res(res);

    // Can not set whitelist if current round is active
    app.set_block(BlockInfo {
        time: rounds.clone()[0].start_time,
        height: 1,
        chain_id: "cosmos".to_string(),
    });
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &OpenEditionMinterExecuteMsg::UpdateWhitelistAddress {
                address: whitelist_address.clone(),
            },
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::WhitelistAlreadyActive {});

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
            &OpenEditionMinterExecuteMsg::UpdateWhitelistAddress {
                address: minter_address.clone(),
            },
            &[],
        )
        .unwrap_err();
    assert!(res
        .source()
        .unwrap()
        .downcast_ref::<OpenEditionMinterError>()
        .is_some());
    // Can not set whitelist if provided address is a already active whitelist
    let mut rounds = return_rounds();
    rounds[0].start_time = Timestamp::from_nanos(1_000);
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
            &OpenEditionMinterExecuteMsg::UpdateWhitelistAddress {
                address: whitelist_address.clone(),
            },
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::WhitelistAlreadyActive {});

    // Create a non active whitelist
    let mut rounds = return_rounds();
    rounds[0].start_time = Timestamp::from_nanos(2_000);
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
            &OpenEditionMinterExecuteMsg::UpdateWhitelistAddress {
                address: whitelist_address.clone(),
            },
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::Unauthorized {});

    // Set whitelist
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &OpenEditionMinterExecuteMsg::UpdateWhitelistAddress {
                address: whitelist_address.clone(),
            },
            &[],
        )
        .unwrap();
    let config: Config = app
        .wrap()
        .query_wasm_smart(
            minter_address.clone(),
            &OpenEditionMinterQueryMsg::Config {},
        )
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
    let open_edition_minter_factory_code_id = res.open_edition_minter_factory_code_id;
    let open_edition_minter_code_id = res.open_edition_minter_code_id;
    let mut app = res.app;

    // Instantiate the oem minter factory
    let open_edition_minter_factory_instantiate_msg =
        return_open_edition_minter_factory_inst_message(open_edition_minter_code_id, None);

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

    // Create an open edition minter
    let open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
    let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateOpenEditionMinter {
        msg: open_edition_minter_instantiate_msg.clone(),
    };
    let public_minting_start_time = open_edition_minter_instantiate_msg.init.start_time;
    let public_minting_price = open_edition_minter_instantiate_msg.init.mint_price.clone();

    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let minter_address = get_contract_address_from_res(res);

    // Burn remaining tokens execution sets num_tokens to 0
    // Try minting before
    app.set_block(BlockInfo {
        time: public_minting_start_time,
        height: 1,
        chain_id: "cosmos".to_string(),
    });
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &OpenEditionMinterExecuteMsg::Mint {},
            &[public_minting_price.clone()],
        )
        .unwrap();

    // Query collection
    let collection = query_onft_collection(&app.storage(), minter_address.clone());
    assert_eq!(collection.onfts.len(), 1);

    // Non creator can not burn remaining tokens
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &OpenEditionMinterExecuteMsg::BurnRemainingTokens {},
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::Unauthorized {});

    // Creator can burn remaining tokens
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &OpenEditionMinterExecuteMsg::BurnRemainingTokens {},
            &[],
        )
        .unwrap();

    // Query Config
    let config: Config = app
        .wrap()
        .query_wasm_smart(
            minter_address.clone(),
            &OpenEditionMinterQueryMsg::Config {},
        )
        .unwrap();
    assert_eq!(config.num_tokens, Some(0));

    // Try minting after
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &OpenEditionMinterExecuteMsg::Mint {},
            &[public_minting_price.clone()],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::NoTokensLeftToMint {});
}

#[test]
fn update_royalty_ratio() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let collector = res.test_accounts.collector;
    let open_edition_minter_factory_code_id = res.open_edition_minter_factory_code_id;
    let open_edition_minter_code_id = res.open_edition_minter_code_id;
    let mut app = res.app;

    // Instantiate the oem minter factory
    let open_edition_minter_factory_instantiate_msg =
        return_open_edition_minter_factory_inst_message(open_edition_minter_code_id, None);

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

    // Create an open edition minter
    let open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
    let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateOpenEditionMinter {
        msg: open_edition_minter_instantiate_msg.clone(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let minter_address = get_contract_address_from_res(res);

    // Send invalid ratio
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &OpenEditionMinterExecuteMsg::UpdateRoyaltyRatio {
                ratio: "One".to_string(),
            },
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(
        error,
        &OpenEditionMinterError::Std(StdError::generic_err("Error parsing whole"))
    );

    // Send invalid ratio
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &OpenEditionMinterExecuteMsg::UpdateRoyaltyRatio {
                ratio: "1/2/3".to_string(),
            },
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(
        error,
        &OpenEditionMinterError::Std(StdError::generic_err("Error parsing whole"))
    );

    // Send more than 100 ratio
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &OpenEditionMinterExecuteMsg::UpdateRoyaltyRatio {
                ratio: "101".to_string(),
            },
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(
        error,
        &OpenEditionMinterError::TokenDetailsError(TokenDetailsError::InvalidRoyaltyRatio {})
    );

    // Send valid ratio with non creator
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &OpenEditionMinterExecuteMsg::UpdateRoyaltyRatio {
                ratio: "50".to_string(),
            },
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::Unauthorized {});

    // Send valid ratio with creator
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &OpenEditionMinterExecuteMsg::UpdateRoyaltyRatio {
                ratio: "0.50".to_string(),
            },
            &[],
        )
        .unwrap();

    // Query token details
    let token: TokenDetails = app
        .wrap()
        .query_wasm_smart(
            minter_address.clone(),
            &OpenEditionMinterQueryMsg::TokenDetails {},
        )
        .unwrap();
    assert_eq!(token.royalty_ratio.to_string(), "0.5");
}

#[test]
fn update_mint_price() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let collector = res.test_accounts.collector;
    let open_edition_minter_factory_code_id = res.open_edition_minter_factory_code_id;
    let open_edition_minter_code_id = res.open_edition_minter_code_id;
    let mut app = res.app;

    // Instantiate the oem minter factory
    let open_edition_minter_factory_instantiate_msg =
        return_open_edition_minter_factory_inst_message(open_edition_minter_code_id, None);

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

    // Create an open edition minter
    let open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
    let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateOpenEditionMinter {
        msg: open_edition_minter_instantiate_msg.clone(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let minter_address = get_contract_address_from_res(res);
    // Non creator can not update mint price
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &OpenEditionMinterExecuteMsg::UpdateMintPrice {
                mint_price: coin(1000000, "uflix"),
            },
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::Unauthorized {});

    // Creator can update mint price
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &OpenEditionMinterExecuteMsg::UpdateMintPrice {
                mint_price: coin(1000000, "uflix"),
            },
            &[],
        )
        .unwrap();

    // Query Config
    let config: Config = app
        .wrap()
        .query_wasm_smart(
            minter_address.clone(),
            &OpenEditionMinterQueryMsg::Config {},
        )
        .unwrap();
    assert_eq!(config.mint_price, coin(1000000, "uflix"));
}
#[test]
fn update_admin() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let collector = res.test_accounts.collector;
    let open_edition_minter_factory_code_id = res.open_edition_minter_factory_code_id;
    let open_edition_minter_code_id = res.open_edition_minter_code_id;
    let mut app = res.app;

    // Instantiate the oem minter factory
    let open_edition_minter_factory_instantiate_msg =
        return_open_edition_minter_factory_inst_message(open_edition_minter_code_id, None);

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

    // Create an open edition minter
    let open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
    let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateOpenEditionMinter {
        msg: open_edition_minter_instantiate_msg.clone(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let minter_address = get_contract_address_from_res(res);

    // Query AuthDetails
    let auth_details: AuthDetails = app
        .wrap()
        .query_wasm_smart(
            minter_address.clone(),
            &OpenEditionMinterQueryMsg::AuthDetails {},
        )
        .unwrap();
    assert_eq!(auth_details.admin, creator);

    // Non creator can not update admin
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &OpenEditionMinterExecuteMsg::UpdateAdmin {
                admin: collector.to_string(),
            },
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::Unauthorized {});

    // Creator can update admin
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &OpenEditionMinterExecuteMsg::UpdateAdmin {
                admin: collector.to_string(),
            },
            &[],
        )
        .unwrap();

    // Query AuthDetails
    let auth_details: AuthDetails = app
        .wrap()
        .query_wasm_smart(
            minter_address.clone(),
            &OpenEditionMinterQueryMsg::AuthDetails {},
        )
        .unwrap();
    assert_eq!(auth_details.admin, collector);
}
#[test]
fn update_payment_collector() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let collector = res.test_accounts.collector;
    let open_edition_minter_factory_code_id = res.open_edition_minter_factory_code_id;
    let open_edition_minter_code_id = res.open_edition_minter_code_id;
    let mut app = res.app;

    // Instantiate the oem minter factory
    let open_edition_minter_factory_instantiate_msg =
        return_open_edition_minter_factory_inst_message(open_edition_minter_code_id, None);

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

    // Create an open edition minter
    let open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
    let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateOpenEditionMinter {
        msg: open_edition_minter_instantiate_msg.clone(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let minter_address = get_contract_address_from_res(res);

    // Query AuthDetails
    let auth_details: AuthDetails = app
        .wrap()
        .query_wasm_smart(
            minter_address.clone(),
            &OpenEditionMinterQueryMsg::AuthDetails {},
        )
        .unwrap();
    assert_eq!(auth_details.payment_collector, creator);

    // Non creator can not update payment collector
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &OpenEditionMinterExecuteMsg::UpdatePaymentCollector {
                payment_collector: collector.to_string(),
            },
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::Unauthorized {});

    // Creator can update payment collector
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &OpenEditionMinterExecuteMsg::UpdatePaymentCollector {
                payment_collector: collector.to_string(),
            },
            &[],
        )
        .unwrap();

    // Query AuthDetails
    let auth_details: AuthDetails = app
        .wrap()
        .query_wasm_smart(
            minter_address.clone(),
            &OpenEditionMinterQueryMsg::AuthDetails {},
        )
        .unwrap();
    assert_eq!(auth_details.payment_collector, collector);
}
