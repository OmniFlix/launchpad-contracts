use cosmwasm_schema::write_api;

use omniflix_round_whitelist::msg::ExecuteMsg;

use whitelist_types::{CreateWhitelistMsg, RoundWhitelistQueryMsgs};

fn main() {
    write_api! {
        instantiate: CreateWhitelistMsg,
        execute: ExecuteMsg,
        query: RoundWhitelistQueryMsgs,
    }
}
