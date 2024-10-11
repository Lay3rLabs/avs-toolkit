use cosmwasm_std::Timestamp;
use lavs_apis::time::{Duration, NANOS_PER_SECONDS};

/// calculator to determine expiration timestamps based on block durations and offsets.
pub struct ExpirationCalculator {
    pub block_duration: Duration,
    pub block_offset: Duration,
}

impl ExpirationCalculator {
    pub fn new(block_duration: Duration, block_offset: Duration) -> Self {
        ExpirationCalculator {
            block_duration,
            block_offset,
        }
    }
    /// calculates the expiration timestamp given based on start time, number of blocks, and timeout in seconds.
    pub fn calculate(
        &self,
        start_time: Timestamp,
        n_blocks: u64,
        timeout_seconds: u64,
    ) -> Timestamp {
        let duration = Duration::new_nanos(
            n_blocks * self.block_duration.as_nanos()
                + self.block_offset.as_nanos()
                + timeout_seconds * NANOS_PER_SECONDS,
        );
        Timestamp::from_nanos(start_time.nanos() + duration.as_nanos())
    }
}
