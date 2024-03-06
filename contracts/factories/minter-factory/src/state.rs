use crate::msg::MinterFactoryParams;
use cw_storage_plus::Item;

pub const PARAMS: Item<MinterFactoryParams> = Item::new("params");
