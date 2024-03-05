use crate::msg::RoundWhitelistFactoryParams;
use cw_storage_plus::Item;

pub const PARAMS: Item<RoundWhitelistFactoryParams> = Item::new("params");
