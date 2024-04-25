use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

use crate::token_details::Token;

#[cw_serde]
pub struct AuthDetails {
    pub admin: Addr,
    pub payment_collector: Addr,
}

#[derive(Default)]
#[cw_serde]
pub struct UserDetails {
    pub minted_tokens: Vec<Token>,
    pub total_minted_count: u32,
    pub public_mint_count: u32,
}
