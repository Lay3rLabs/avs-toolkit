use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdResult, Uint64};
use cw_storage_plus::{IntKey, Key, KeyDeserialize, Prefixer, PrimaryKey};
use std::{hash::Hash, num::ParseIntError, str::FromStr};

/// An id for a task. This is a simple wrapper around a `Uint64` internally.
/// Serialized on the wire as a string
#[cw_serde]
#[derive(Copy, PartialOrd, Ord, Eq)]
pub struct TaskId(Uint64);

impl Hash for TaskId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.u64().hash(state);
    }
}

impl TaskId {
    /// Construct a new value from a [u64].
    pub fn new(x: u64) -> Self {
        Self(x.into())
    }

    /// The underlying `u64` representation.
    pub fn u64(self) -> u64 {
        self.0.u64()
    }
}

impl<'a> PrimaryKey<'a> for TaskId {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = Self;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        vec![Key::Val64(self.0.u64().to_cw_bytes())]
    }
}

impl<'a> Prefixer<'a> for TaskId {
    fn prefix(&self) -> Vec<Key> {
        vec![Key::Val64(self.0.u64().to_cw_bytes())]
    }
}

impl KeyDeserialize for TaskId {
    type Output = TaskId;
    const KEY_ELEMS: u16 = 1;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        u64::from_vec(value).map(|x| TaskId(Uint64::new(x)))
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TaskId {
    type Err = ParseIntError;
    fn from_str(src: &str) -> Result<Self, ParseIntError> {
        src.parse().map(|x| TaskId(Uint64::new(x)))
    }
}
