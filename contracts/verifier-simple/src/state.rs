use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Env, StdError, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use lavs_apis::verifier_simple::TaskStatus;

pub const CONFIG: Item<Config> = Item::new("config");

pub const TASKS: Map<(&Addr, u64), TaskMetadata> = Map::new("tasks");
pub const OPTIONS: Map<(&Addr, u64, &str), TaskOption> = Map::new("task_options");
pub const VOTES: Map<(&Addr, u64, &Addr), OperatorVote> = Map::new("operator_votes");

#[cw_serde]
pub struct Config {
    pub operators: Addr,
    pub required_percentage: u32,
}

/// Metadata for a task - indexed by (task_queue, task_id)
#[cw_serde]
pub struct TaskMetadata {
    pub power_required: Uint128,
    pub status: TaskStatus,
    pub created_height: u64,
    /// Measured in UNIX seconds
    pub expires_time: u64,
}

impl TaskMetadata {
    pub fn is_expired(&self, env: &Env) -> bool {
        env.block.time.seconds() >= self.expires_time
    }
}

/// Metadata for a task option with some votes - indexed by (task_queue, task_id, result)
#[cw_serde]
pub struct TaskOption {
    pub power: Uint128,
}

/// Metadata for a given vote by an operator - indexed by (task_queue, task_id, operator)
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
    task_id: u64,
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