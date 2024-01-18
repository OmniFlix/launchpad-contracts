use cosmwasm_schema::write_api;

use open_edition_minter_types::{InstantiateMsg, QueryMsg};

use omniflix_open_edition_minter::msg::ExecuteMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
