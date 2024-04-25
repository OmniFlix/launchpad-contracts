use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;
use minter_types::token_details::Token;
use omniflix_std::types::omniflix::onft::v1beta1::WeightedAddress;

#[cw_serde]
pub enum ExecuteMsg {
    Mint {},
    MintAdmin {
        recipient: String,
        token_id: Option<String>,
    },
    BurnRemainingTokens {},
    UpdateRoyaltyRatio {
        ratio: String,
    },
    UpdateMintPrice {
        mint_price: Coin,
    },
    RandomizeList {},
    UpdateWhitelistAddress {
        address: String,
    },
    Pause {},
    Unpause {},
    // This directly updates the pausers list if the sender is one of the pausers
    // At every update full list of pausers should be sent
    SetPausers {
        pausers: Vec<String>,
    },
    UpdateRoyaltyReceivers {
        receivers: Vec<WeightedAddress>,
    },
    UpdateDenom {
        collection_name: Option<String>,
        description: Option<String>,
        preview_uri: Option<String>,
    },
    PurgeDenom {},
    UpdateAdmin {
        admin: String,
    },
    UpdatePaymentCollector {
        payment_collector: String,
    },
}
#[cw_serde]
#[derive(QueryResponses)]
pub enum MinterExtensionQueryMsg {
    #[returns(Vec<Token>)]
    MintableTokens {},
    #[returns(u32)]
    TotalTokensRemaining {},
}
