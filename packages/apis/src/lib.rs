use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Timestamp, Uint64};

pub mod id;
pub mod interfaces;
pub mod tasks;
pub mod verifier_simple;

/// A wrapper around u64, that represents duration between two points in time.
#[cw_serde]
#[derive(Copy, PartialOrd, Ord, Eq)]
pub struct Nanos(Uint64);

impl Nanos {
    /// Create a new `Nanos` from `u64` value.
    pub fn new(input: u64) -> Self {
        Self(input.into())
    }

    /// Get a copy of the internal data
    pub fn u64(self) -> u64 {
        self.0.u64()
    }
}

impl std::fmt::Display for Nanos {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
