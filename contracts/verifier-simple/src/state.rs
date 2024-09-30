use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdError, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use lavs_apis::{id::TaskId, verifier_simple::TaskMetadata};

pub const CONFIG: Item<Config> = Item::new("config");

// key is (task_queue_address, task_id)
pub const TASKS: Map<(&Addr, TaskId), TaskMetadata> = Map::new("tasks");
// key is (task_queue_address, task_id, result)
pub const OPTIONS: Map<(&Addr, TaskId, &str), TaskOption> = Map::new("task_options");
/// key is (task_queue_address, task_id, operator)
pub const VOTES: Map<(&Addr, TaskId, &Addr), OperatorVote> = Map::new("operator_votes");

#[cw_serde]
pub struct Config {
    pub operators: Addr,
    pub required_percentage: u32,
}

/// Metadata for a task option with some votes - indexed by (task_queue, task_id, result)
#[cw_serde]
pub struct TaskOption {
    pub power: Uint128,
}

#[cw_serde]
pub struct OperatorVote {
    pub power: Uint128,
    pub result: String,
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
    let vote = OperatorVote {
        power,
        result: result.to_string(),
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
