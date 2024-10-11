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

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{BlockInfo, Timestamp};
    use serde_json::{from_str, to_string};

    #[test]
    fn create_expiration_from_duration() {
        let duration = Duration::new(33);
        let block_info = BlockInfo {
            height: 1,
            time: Timestamp::from_seconds(66),
            chain_id: "id".to_owned(),
        };
        assert_eq!(
            duration.after(&block_info),
            Expiration::at_timestamp(Timestamp::from_seconds(99))
        );
    }

    #[test]
    fn expiration_has_expired() {
        let expiration = Expiration::at_timestamp(Timestamp::from_seconds(10));
        let block_info = BlockInfo {
            height: 1,
            time: Timestamp::from_seconds(9),
            chain_id: "id".to_owned(),
        };
        assert!(!expiration.is_expired(&block_info));
        let block_info = BlockInfo {
            height: 1,
            time: Timestamp::from_seconds(10),
            chain_id: "id".to_owned(),
        };
        assert!(expiration.is_expired(&block_info));
        let block_info = BlockInfo {
            height: 1,
            time: Timestamp::from_seconds(11),
            chain_id: "id".to_owned(),
        };
        assert!(expiration.is_expired(&block_info));
    }

    #[test]
    fn duration_serialization_as_integer_string() {
        let duration = Duration::new(42);
        let json_duration = to_string(&duration).unwrap();
        assert_eq!(json_duration, "\"42\"");

        let deserialized_duration: Duration = from_str(&json_duration).unwrap();
        assert_eq!(deserialized_duration.seconds(), 42);
    }

    #[test]
    fn expiration_serialization_as_timestamp() {
        let expiration = Expiration::at_timestamp(Timestamp::from_seconds(1000));
        let json_expiration = to_string(&expiration).unwrap();
        // because of serialization to nanoseconds
        assert_eq!(json_expiration, "\"1000000000000\"");

        let deserialized_expiration: Expiration = from_str(&json_expiration).unwrap();
        assert_eq!(
            deserialized_expiration.time(),
            Timestamp::from_seconds(1000)
        );
    }

    #[test]
    fn duration_and_expiration_in_struct_serialization() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct TestStruct {
            duration: Duration,
            expiration: Expiration,
        }

        let test_struct = TestStruct {
            duration: Duration::new(3600),
            expiration: Expiration::at_timestamp(Timestamp::from_seconds(10_000)),
        };

        let json_struct = to_string(&test_struct).unwrap();
        assert_eq!(
            json_struct,
            r#"{"duration":"3600","expiration":"10000000000000"}"#
        );

        let deserialized_struct: TestStruct = from_str(&json_struct).unwrap();
        assert_eq!(deserialized_struct, test_struct);
    }
}
