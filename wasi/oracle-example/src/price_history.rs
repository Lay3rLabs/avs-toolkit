use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const PRICE_HISTORY_FILE_PATH: &str = "price_history.json";

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AveragePrice {
    pub price: f32,
    pub count: usize,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PriceHistory {
    pub btcusd_prices: VecDeque<(u64, f32)>,
}

impl PriceHistory {
    /// Read price history from the file system or initialize empty.
    pub fn read() -> Result<Self, String> {
        match std::fs::read(PRICE_HISTORY_FILE_PATH) {
            Ok(bytes) => {
                serde_json::from_slice::<PriceHistory>(&bytes).map_err(|err| err.to_string())
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Default::default()),
            Err(err) => Err(err.to_string()),
        }
    }

    /// Record latest price to price history and truncate to max of 1000 of price history.
    /// `now` is the specified time in UNIX Epoch seconds.
    ///
    /// Updates the price history on the file system.
    pub fn record_latest_price(&mut self, now: u64, price: f32) -> Result<(), String> {
        // add to the front of the list
        self.btcusd_prices.push_front((now, price));
        self.btcusd_prices.truncate(1000);

        // write price history
        std::fs::write(
            PRICE_HISTORY_FILE_PATH,
            serde_json::to_vec(&self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())
    }

    /// Calculate the average price since the specified time in UNIX Epoch seconds.
    pub fn average(&self, since_time_secs: u64) -> AveragePrice {
        let mut sum = 0f64;
        let mut count = 0;
        for (t, p) in self.btcusd_prices.iter() {
            if t >= &since_time_secs {
                sum += *p as f64;
                count += 1;
            } else {
                break;
            }
        }
        AveragePrice {
            price: (sum / (count as f64)) as f32,
            count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_average_prices() {
        let history = PriceHistory {
            btcusd_prices: VecDeque::from([
                (20, 15.0),
                (14, 20.0),
                (10, 10.0),
                (1, 10.0),
                (0, 10.0),
            ]),
        };

        assert_eq!(
            history.average(10),
            AveragePrice {
                price: 15.0,
                count: 3,
            }
        );
        assert_eq!(
            history.average(0),
            AveragePrice {
                price: 13.0,
                count: 5,
            }
        );
    }
}
