pub mod contract;
mod error;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;

/// This is used for cw-orch
#[cfg(not(target_arch = "wasm32"))]
pub mod interface;

#[cfg(test)]
pub mod tests;
