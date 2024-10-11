use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint64;

pub const NANOS_PER_SECONDS: u64 = 1_000_000_000;

/// A wrapper around u64, that represents duration between two points in time.
#[cw_serde]
#[derive(Copy, PartialOrd, Ord, Eq)]
pub struct Duration(Uint64);

impl Duration {
    pub fn new_nanos(nanos: u64) -> Duration {
        Duration(nanos.into())
    }

    pub fn new_seconds(seconds: u64) -> Duration {
        Duration((seconds * NANOS_PER_SECONDS).into())
    }

    pub fn as_nanos(&self) -> u64 {
        self.0.u64()
    }

    pub fn as_seconds(&self) -> u64 {
        self.0.u64() / NANOS_PER_SECONDS
    }
}

impl std::fmt::Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
