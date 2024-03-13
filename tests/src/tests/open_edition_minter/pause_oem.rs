#![cfg(test)]
use cosmwasm_std::{coin, Addr, BlockInfo};

use cw_multi_test::Executor;
use minter_types::types::Config;
use omniflix_open_edition_minter_factory::msg::ExecuteMsg as OpenEditionMinterFactoryExecuteMsg;
use pauser::PauseError;

use crate::helpers::utils::get_contract_address_from_res;

use crate::helpers::mock_messages::factory_mock_messages::return_open_edition_minter_factory_inst_message;

use crate::helpers::mock_messages::oem_mock_messages::return_open_edition_minter_inst_msg;

use crate::helpers::setup::setup;
use omniflix_open_edition_minter::msg::OEMQueryExtension;

use minter_types::msg::QueryMsg as BaseMinterQueryMsg;

use omniflix_open_edition_minter::msg::ExecuteMsg as OpenEditionMinterExecuteMsg;

use omniflix_open_edition_minter::error::ContractError as OpenEditionMinterError;

type OpenEditionMinterQueryMsg = BaseMinterQueryMsg<OEMQueryExtension>;

#[test]
fn test_pause_oem() {
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

    let open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
    let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateOpenEditionMinter {
        msg: open_edition_minter_instantiate_msg,
    };
    // Create an open edition minter
    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();

    let oem_contract_address = get_contract_address_from_res(res);

    // Ensure the oem minter is not paused
    let is_paused: bool = app
        .wrap()
        .query_wasm_smart(
            &oem_contract_address,
            &OpenEditionMinterQueryMsg::IsPaused {},
        )
        .unwrap();
    assert_eq!(is_paused, false);

    // Pause only affects minting
    // Try to mint a token
    // Set time to public minting time
    let config: Config = app
        .wrap()
        .query_wasm_smart(&oem_contract_address, &OpenEditionMinterQueryMsg::Config {})
        .unwrap();
    let public_minting_time = config.start_time;
    app.set_block(BlockInfo {
        time: public_minting_time,
        height: 1,
        chain_id: "".to_string(),
    });
    // Mint a token
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(oem_contract_address.clone()),
            &OpenEditionMinterExecuteMsg::Mint {},
            &[coin(1000000, "uflix")],
        )
        .unwrap();

    // Pause with non pauser
    let error = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(oem_contract_address.clone()),
            &OpenEditionMinterFactoryExecuteMsg::Pause {},
            &[],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(
        error,
        &OpenEditionMinterError::Pause(PauseError::Unauthorized {
            sender: collector.clone()
        })
    );

    // Pause the oem minter
    let _res = app.execute_contract(
        creator.clone(),
        Addr::unchecked(oem_contract_address.clone()),
        &OpenEditionMinterFactoryExecuteMsg::Pause {},
        &[],
    );

    let is_paused: bool = app
        .wrap()
        .query_wasm_smart(
            &oem_contract_address,
            &OpenEditionMinterQueryMsg::IsPaused {},
        )
        .unwrap();
    assert_eq!(is_paused, true);

    // Attempt to mint a token
    let error = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(oem_contract_address.clone()),
            &OpenEditionMinterExecuteMsg::Mint {},
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::Pause(PauseError::Paused {}));

    // Attempt to mint a token with creator
    let error = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(oem_contract_address.clone()),
            &OpenEditionMinterExecuteMsg::MintAdmin {
                recipient: creator.clone().into_string(),
            },
            &[],
        )
        .unwrap_err();

    let res = error.source().unwrap();
    let error = res.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::Pause(PauseError::Paused {}));
}
