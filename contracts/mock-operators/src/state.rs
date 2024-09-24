use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct Config {
    pub operators: Vec<OpInfo>,
    pub total_power: Uint128,
}

#[cw_serde]
pub struct OpInfo {
    pub op: Addr,
    pub power: Uint128,
}
