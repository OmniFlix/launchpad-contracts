use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub enum ExecuteMsg {
    Mint {},
    MintAdmin { recipient: String },
    UpdateRoyaltyRatio { ratio: String },
    UpdateMintPrice { mint_price: Uint128 },
    UpdateWhitelistAddress { address: String },
    Pause {},
    Unpause {},
    SetPausers { pausers: Vec<String> },
}
