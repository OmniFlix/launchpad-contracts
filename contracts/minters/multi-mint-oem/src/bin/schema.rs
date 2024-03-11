use cosmwasm_schema::write_api;

use minter_types::msg::QueryMsg;

use omniflix_multi_mint_open_edition_minter::msg::{ExecuteMsg, QueryMsgExtension};

use omniflix_open_edition_minter_factory::msg::OpenEditionMinterCreateMsg;

fn main() {
    write_api! {
        instantiate: OpenEditionMinterCreateMsg,
        execute: ExecuteMsg,
        query: QueryMsg<QueryMsgExtension>,
    }
}
