use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub enum ExecuteMsg {
    Mint {},
    MintAdmin {
        recipient: String,
        denom_id: Option<String>,
    },
    BurnRemainingTokens {},
    UpdateRoyaltyRatio {
        ratio: String,
    },
    UpdateMintPrice {
        mint_price: Uint128,
    },
    RandomizeList {},
    UpdateWhitelistAddress {
        address: String,
    },
}
