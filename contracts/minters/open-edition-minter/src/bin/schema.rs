use cosmwasm_schema::write_api;

use omniflix_open_edition_minter::msg::{ExecuteMsg, OEMQueryExtension};

use minter_types::QueryMsg;

use omniflix_open_edition_minter_factory::msg::OpenEditionMinterCreateMsg;

fn main() {
    write_api! {
        instantiate: OpenEditionMinterCreateMsg,
        execute: ExecuteMsg,
        query: QueryMsg<OEMQueryExtension>,
    }
}
