use cw_orch::{interface, prelude::*};

use crate::msg::{ExecuteMsg, QueryMsg};
type InstantiateMsg = cosmwasm_std::Empty;
type MigrateMsg = cosmwasm_std::Empty;

pub const CONTRACT_ID: &str = env!("CARGO_PKG_NAME");

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg, id = CONTRACT_ID)]
pub struct Contract;

impl<Chain> Uploadable for Contract<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path(CONTRACT_ID)
            .unwrap()
    }

    /// Returns a CosmWasm contract wrapper
    fn wrapper() -> Box<dyn MockContract<Empty>> {
        Box::new(
            ContractWrapper::new_with_empty(
                crate::contract::execute,
                crate::contract::instantiate,
                crate::contract::query,
            ), // .with_migrate(crate::contract::migrate),
        )
    }
}
