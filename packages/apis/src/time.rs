use cosmwasm_schema::cw_serde;
use cosmwasm_std::{BlockInfo, Timestamp, Uint64};

/// A wrapper around u64, that represents duration between two points in time.
#[cw_serde]
#[derive(Copy, PartialOrd, Ord, Eq)]
pub struct Duration(Uint64);

impl Duration {
    pub fn new(secs: u64) -> Duration {
        Duration(secs.into())
    }

    pub fn after(&self, block: &BlockInfo) -> Expiration {
        self.after_time(block.time)
    }

    pub fn after_time(&self, timestamp: Timestamp) -> Expiration {
        Expiration::at_timestamp(timestamp.plus_seconds(self.0.into()))
    }

    pub fn seconds(&self) -> u64 {
        self.0.u64()
    }
}

impl std::fmt::Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cw_serde]
#[derive(Copy, PartialOrd, Ord, Eq)]
pub struct Expiration(Timestamp);

impl Expiration {
    pub fn now(block: &BlockInfo) -> Self {
        Self(block.time)
    }

    pub fn at_timestamp(timestamp: Timestamp) -> Self {
        Self(timestamp)
    }

    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        self.is_expired_time(block.time)
    }

    pub fn is_expired_time(&self, timestamp: Timestamp) -> bool {
        timestamp >= self.0
    }

    pub fn time(&self) -> Timestamp {
        self.0
    }

    pub fn as_key(&self) -> u64 {
        self.0.nanos()
    }
}
impl From<Expiration> for Timestamp {
    fn from(expiration: Expiration) -> Timestamp {
        expiration.0
    }
}
