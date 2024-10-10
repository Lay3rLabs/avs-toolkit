use std::str::FromStr;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, StdError, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use lavs_apis::{id::TaskId, verifier_simple::TaskMetadata};

pub const CONFIG: Item<Config> = Item::new("config");
pub const VOTES: Map<(&Addr, TaskId, &Addr), OperatorVote> = Map::new("operator_votes");
pub const TASKS: Map<(&Addr, TaskId), TaskMetadata> = Map::new("tasks");
pub const OPTIONS: Map<(&Addr, TaskId, &str), TaskOption> = Map::new("task_options");
pub const SLASHED_OPERATORS: Map<&Addr, bool> = Map::new("slashed_operators");

#[cw_serde]
pub struct Config {
    pub operator_contract: Addr,
    pub threshold_percent: Decimal,
    pub allowed_spread: Decimal,
    pub slashable_spread: Decimal,
    pub required_percentage: u32,
}

#[cw_serde]
pub struct OperatorVote {
    pub power: Uint128,
    pub result: Decimal,
}

/// Metadata for a task option with some votes - indexed by (task_queue, task_id, result)
#[cw_serde]
pub struct TaskOption {
    pub power: Uint128,
}

#[cw_serde]
pub struct PriceResult {
    pub price: String,
}

/// This assumes a previous check was made that the operator has not yet voted.
/// Returns the running tally of votes in favor of this result.
pub fn record_vote(
    storage: &mut dyn Storage,
    task_queue: &Addr,
    task_id: TaskId,
    operator: &Addr,
    result: &str,
    power: Uint128,
) -> Result<Uint128, StdError> {
    let price_result: PriceResult =
        serde_json::from_str(result).expect("Record vote: Invalid result input provided");

    let vote = OperatorVote {
        power,
        result: Decimal::from_str(&price_result.price)?,
    };

    VOTES
        .save(storage, (task_queue, task_id, operator), &vote)
        .unwrap();

    // Update the option and get the running tally of power in favor of this result
    let tally = OPTIONS.update::<_, StdError>(storage, (task_queue, task_id, result), |old| {
        let old_power = old.map_or(Uint128::zero(), |v| v.power);
        Ok(TaskOption {
            power: old_power + power,
        })
    })?;
    Ok(tally.power)
}
