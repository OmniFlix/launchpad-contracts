#![cfg(test)]
use cosmwasm_std::coin;

use cw_multi_test::Executor;
use omniflix_open_edition_minter_factory::msg::ExecuteMsg as OpenEditionMinterFactoryExecuteMsg;
use pauser::PauseError;

use crate::helpers::mock_messages::factory_mock_messages::return_open_edition_minter_factory_inst_message;

use crate::helpers::mock_messages::oem_mock_messages::return_open_edition_minter_inst_msg;

use crate::helpers::setup::setup;

use omniflix_open_edition_minter_factory::error::ContractError as OpenEditionMinterFactoryError;
use omniflix_open_edition_minter_factory::msg::QueryMsg as OpenEditionMinterFactoryQueryMsg;

#[test]
fn paused_oem_factory() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
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

    // Ensure the factory is not paused
    let is_paused: bool = app
        .wrap()
        .query_wasm_smart(
            &open_edition_minter_factory_address,
            &OpenEditionMinterFactoryQueryMsg::IsPaused {},
        )
        .unwrap();
    assert!(!is_paused);

    // Create an open edition minter
    let open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
    let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateOpenEditionMinter {
        msg: open_edition_minter_instantiate_msg,
    };

    let _res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    // Pause the factory non admin
    let error = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &OpenEditionMinterFactoryExecuteMsg::Pause {},
            &[],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<OpenEditionMinterFactoryError>().unwrap();
    assert_eq!(
        error,
        &OpenEditionMinterFactoryError::Pause(PauseError::Unauthorized {
            sender: creator.clone()
        })
    );

    // Pause the factory
    let _res = app.execute_contract(
        admin.clone(),
        open_edition_minter_factory_address.clone(),
        &OpenEditionMinterFactoryExecuteMsg::Pause {},
        &[],
    );

    let is_paused: bool = app
        .wrap()
        .query_wasm_smart(
            &open_edition_minter_factory_address,
            &OpenEditionMinterFactoryQueryMsg::IsPaused {},
        )
        .unwrap();
    assert!(is_paused);

    // Attempt to create an open edition minter
    let open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
    let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateOpenEditionMinter {
        msg: open_edition_minter_instantiate_msg,
    };

    let error = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<OpenEditionMinterFactoryError>().unwrap();
    assert_eq!(
        error,
        &OpenEditionMinterFactoryError::Pause(PauseError::Paused {})
    );
}
