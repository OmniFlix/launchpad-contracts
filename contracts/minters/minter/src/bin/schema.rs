use cosmwasm_schema::write_api;

use omniflix_minter::msg::ExecuteMsg;

use minter_types::QueryMsg;

use omniflix_minter_factory::msg::CreateMinterMsg;

fn main() {
    write_api! {
        instantiate: CreateMinterMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
