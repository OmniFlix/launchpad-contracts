use cosmwasm_std::{coins, Addr, BlockInfo, Timestamp};
use cw_multi_test::ContractWrapper;
use omniflix_minter::contract::{
    execute as minter_execute, instantiate as minter_instantiate, query as minter_query,
};
use omniflix_minter_factory::contract::{
    execute as factory_execute, instantiate as factory_instantiate, query as factory_query,
};
use omniflix_open_edition_minter::contract::{
    execute as open_edition_minter_execute, instantiate as open_edition_minter_instantiate,
    query as open_edition_minter_query,
};

use crate::helpers::utils::mint_to_address;
use omniflix_open_edition_minter_factory::contract::{
    execute as open_edition_minter_factory_execute,
    instantiate as open_edition_minter_factory_instantiate,
    query as open_edition_minter_factory_query,
};
use omniflix_round_whitelist::contract::{
    execute as round_whitelist_execute, instantiate as round_whitelist_instantiate,
    query as round_whitelist_query,
};
use omniflix_round_whitelist_factory::contract::{
    execute as round_whitelist_factory_execute, instantiate as round_whitelist_factory_instantiate,
    query as round_whitelist_factory_query,
};

use omniflix_multi_mint_open_edition_minter::contract::{
    execute as multi_mint_open_edition_minter_execute,
    instantiate as multi_mint_open_edition_minter_instantiate,
    query as multi_mint_open_edition_minter_query,
};

use omniflix_testing::app::OmniflixApp;
pub struct TestAdresses {
    pub admin: Addr,
    pub creator: Addr,
    pub collector: Addr,
}

pub fn setup() -> (OmniflixApp, TestAdresses, u64, u64, u64, u64, u64, u64, u64) {
    let mut app = OmniflixApp::new();
    let admin = Addr::unchecked("admin");
    let creator = Addr::unchecked("creator");
    let collector = Addr::unchecked("collector");

    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(1_000),
    });
    mint_to_address(&mut app, admin.to_string(), coins(1000000000, "uflix"));
    mint_to_address(&mut app, creator.to_string(), coins(1000000000, "uflix"));
    mint_to_address(&mut app, collector.to_string(), coins(1000000000, "uflix"));
    mint_to_address(
        &mut app,
        collector.to_string(),
        coins(1000000000000, "diffirent_denom"),
    );
    mint_to_address(
        &mut app,
        collector.to_string(),
        coins(1000000000000, "incorrect_denom"),
    );
    mint_to_address(
        &mut app,
        creator.to_string(),
        coins(1000000000000, "incorrect_denom"),
    );
    mint_to_address(
        &mut app,
        creator.to_string(),
        coins(1000000000000, "diffirent_denom"),
    );

    let minter_factory_contract = Box::new(ContractWrapper::new(
        factory_execute,
        factory_instantiate,
        factory_query,
    ));
    let minter_contract = Box::new(ContractWrapper::new(
        minter_execute,
        minter_instantiate,
        minter_query,
    ));

    let round_whitelist_factory_contract = Box::new(ContractWrapper::new(
        round_whitelist_factory_execute,
        round_whitelist_factory_instantiate,
        round_whitelist_factory_query,
    ));
    let round_whitelist_contract = Box::new(ContractWrapper::new(
        round_whitelist_execute,
        round_whitelist_instantiate,
        round_whitelist_query,
    ));
    let open_edition_minter_factory_contract = Box::new(ContractWrapper::new(
        open_edition_minter_factory_execute,
        open_edition_minter_factory_instantiate,
        open_edition_minter_factory_query,
    ));
    let open_edition_minter_contract = Box::new(ContractWrapper::new(
        open_edition_minter_execute,
        open_edition_minter_instantiate,
        open_edition_minter_query,
    ));
    let multi_mint_open_edition_minter_contract = Box::new(ContractWrapper::new(
        multi_mint_open_edition_minter_execute,
        multi_mint_open_edition_minter_instantiate,
        multi_mint_open_edition_minter_query,
    ));

    let minter_code_id = app.store_code(minter_contract);

    let minter_factory_code_id = app.store_code(minter_factory_contract);

    let round_whitelist_code_id = app.store_code(round_whitelist_contract);

    let round_whitelist_factory_code_id = app.store_code(round_whitelist_factory_contract);

    let open_edition_minter_code_id = app.store_code(open_edition_minter_contract);

    let open_edition_minter_factory_code_id = app.store_code(open_edition_minter_factory_contract);

    let multi_mint_open_edition_minter_code_id =
        app.store_code(multi_mint_open_edition_minter_contract);
    (
        app,
        TestAdresses {
            admin,
            creator,
            collector,
        },
        minter_factory_code_id,
        minter_code_id,
        round_whitelist_factory_code_id,
        round_whitelist_code_id,
        open_edition_minter_factory_code_id,
        open_edition_minter_code_id,
        multi_mint_open_edition_minter_code_id,
    )
}
