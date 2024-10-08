use cosmwasm_std::Uint128;
use cw_storage_plus::Item;

pub const CREATED_COUNT: Item<Uint128> = Item::new("created_count");
