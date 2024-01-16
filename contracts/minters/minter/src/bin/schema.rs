use cosmwasm_schema::write_api;

use omniflix_minter::msg::ExecuteMsg;

use minter_types::{InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
