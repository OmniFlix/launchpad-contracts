use cosmwasm_schema::write_api;

use omniflix_minter::msg::{ExecuteMsg, MinterExtensionQueryMsg};

use minter_types::msg::QueryMsg;

use omniflix_minter_factory::msg::CreateMinterMsgs;

fn main() {
    write_api! {
        instantiate: CreateMinterMsgs,
        execute: ExecuteMsg,
        query: QueryMsg<MinterExtensionQueryMsg>,
    }
}
