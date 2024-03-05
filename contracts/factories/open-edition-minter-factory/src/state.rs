use cw_storage_plus::Item;

use crate::msg::OpenEditionMinterFactoryParams;

pub const PARAMS: Item<OpenEditionMinterFactoryParams> = Item::new("params");
