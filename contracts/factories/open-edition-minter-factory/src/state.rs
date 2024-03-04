use cw_storage_plus::Item;
use factory_types::FactoryParams;

use crate::msg::MultiMinterFactoryExtension;

pub const PARAMS: Item<FactoryParams<MultiMinterFactoryExtension>> = Item::new("params");
