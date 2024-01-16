use cosmwasm_schema::write_api;

use omniflix_round_whitelist::msg::ExecuteMsg;

use whitelist_types::{InstantiateMsg, RoundWhitelistQueryMsgs};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: RoundWhitelistQueryMsgs,
    }
}
