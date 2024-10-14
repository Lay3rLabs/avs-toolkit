use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint64;

const NANOS_PER_SECOND: u64 = 1_000_000_000;

/// A wrapper around u64, that represents duration between two points in time.
#[cw_serde]
#[derive(Copy, PartialOrd, Ord, Eq)]
pub struct Duration(Uint64);

impl Duration {
    pub fn new_nanos(nanos: u64) -> Duration {
        Duration(nanos.into())
    }

    pub fn new_seconds(seconds: u64) -> Duration {
        Duration((seconds * NANOS_PER_SECOND).into())
    }

    pub fn as_nanos(&self) -> u64 {
        self.0.u64()
    }

    pub fn as_seconds(&self) -> u64 {
        self.0.u64() / NANOS_PER_SECOND
    }
}

impl std::fmt::Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn duration_serialization() {
        // 123 seconds/nanos with precision
        let duration_nanos = Duration::new_nanos(1_234_567_890);
        let duration_seconds = Duration::new_seconds(123);

        let serialized_nanos = serde_json::to_string(&duration_nanos).unwrap();
        let serialized_seconds = serde_json::to_string(&duration_seconds).unwrap();

        // seconds has a lot of `0` because of the way it is serialized
        assert_eq!(serialized_nanos, "\"1234567890\"");
        assert_eq!(serialized_seconds, "\"123000000000\"");

        let deserialized_nanos: Duration = serde_json::from_str(&serialized_nanos).unwrap();
        let deserialized_seconds: Duration = serde_json::from_str(&serialized_seconds).unwrap();

        assert_eq!(deserialized_nanos.as_nanos(), 1_234_567_890);
        assert_eq!(deserialized_seconds.as_nanos(), 123_000_000_000);
        assert_eq!(deserialized_seconds.as_seconds(), 123);
    }

    #[test]
    fn duration_zero_serialization() {
        let duration_zero = Duration::new_nanos(0);
        let serialized_zero = serde_json::to_string(&duration_zero).unwrap();
        assert_eq!(serialized_zero, "\"0\"");

        let deserialized_zero: Duration = serde_json::from_str(&serialized_zero).unwrap();
        assert_eq!(deserialized_zero.as_nanos(), 0);
    }

    #[test]
    fn duration_max_value_serialization() {
        let duration_max = Duration::new_nanos(u64::MAX);
        let serialized_max = serde_json::to_string(&duration_max).unwrap();
        assert_eq!(serialized_max, format!("\"{}\"", u64::MAX));

        let deserialized_max: Duration = serde_json::from_str(&serialized_max).unwrap();
        assert_eq!(deserialized_max.as_nanos(), u64::MAX);
        assert_eq!(deserialized_max.as_seconds(), u64::MAX / NANOS_PER_SECOND);
    }

    #[test]
    fn duration_methods() {
        let duration = Duration::new_seconds(2);
        assert_eq!(duration.as_nanos(), 2_000_000_000);
        assert_eq!(duration.as_seconds(), 2);

        let duration_nanos = Duration::new_nanos(2_500_000_000);
        assert_eq!(duration_nanos.as_nanos(), 2_500_000_000);
        // it'll round down
        assert_eq!(duration_nanos.as_seconds(), 2);
    }

    #[test]
    fn duration_equality() {
        let duration1 = Duration::new_nanos(1_000_000_000);
        let duration2 = Duration::new_seconds(1);
        assert_eq!(duration1, duration2);
    }

    #[test]
    fn duration_ordering() {
        let duration_short = Duration::new_nanos(1_000_000_000);
        let duration_long = Duration::new_nanos(2_000_000_000);
        assert!(duration_short < duration_long);
    }
}
