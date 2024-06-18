use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cosmwasm_std::Deps;
use cosmwasm_std::StdError;

use crate::token_details::Token;

#[cw_serde]
pub struct AuthDetails {
    pub admin: Addr,
    pub payment_collector: Addr,
}

impl AuthDetails {
    pub fn validate(&self, deps: &Deps) -> Result<(), StdError> {
        deps.api.addr_validate(self.admin.as_ref())?;
        deps.api
            .addr_validate(self.payment_collector.as_ref())?;
        Ok(())
    }
}

#[derive(Default)]
#[cw_serde]
pub struct UserDetails {
    pub minted_tokens: Vec<Token>,
    pub total_minted_count: u32,
    pub public_mint_count: u32,
}
