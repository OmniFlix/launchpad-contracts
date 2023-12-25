use cosmwasm_schema::write_api;

use omniflix_minter::msg::{ExecuteMsg, QueryMsg};

use minter_types::InstantiateMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
