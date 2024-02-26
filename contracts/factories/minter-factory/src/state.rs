use cosmwasm_std::Empty;
use cw_storage_plus::Item;
use factory_types::FactoryParams;

pub const PARAMS: Item<FactoryParams<Empty>> = Item::new("params");
