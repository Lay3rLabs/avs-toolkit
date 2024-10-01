use cosmwasm_std::{Addr, Decimal, DepsMut, Env, StdError, Uint128};
use cw_utils::PaymentError;
use lavs_apis::{
    id::TaskId,
    interfaces::{
        tasks::TasksStorage,
        voting::{TotalPowerResponse, VotingPowerResponse},
    },
    tasks::{TaskQueryMsg, TaskStatus, TaskStatusResponse},
    verifier_simple::TaskMetadata,
};

use lavs_apis::interfaces::voting::QueryMsg as OperatorQueryMsg;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VerifierError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("Invalid percentage, must be between 1 and 100")]
    InvalidPercentage,

    #[error("Operator tried to vote twice: {0}")]
    OperatorAlreadyVoted(String),

    #[error("Task expired. Cannot vote on it")]
    TaskExpired,

    #[error("Task already completed. Cannot vote on it")]
    TaskAlreadyCompleted,

    #[error("Unauthorized")]
    Unauthorized,
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}

/// Does all checks to ensure the voter is valid and has not voted yet.
/// Also checks the task is valid and still open.
/// Returns the metadata for the task (creating it if first voter), along with the voting power of this operator.
///
/// We do not want to error if an operator votes for a task that is already completed (due to race conditions).
/// In that case, just return None and exit early rather than error.
#[allow(clippy::too_many_arguments)]
pub fn ensure_valid_vote(
    mut deps: DepsMut,
    env: &Env,
    task_queue: &Addr,
    task_id: TaskId,
    operator: &Addr,
    fraction_percent: u32,
    operators_addr: &Addr,
) -> Result<Option<(TaskMetadata, Uint128)>, VerifierError> {
    // Load task info, or create it if not there
    // Error here means the contract is in expired or completed, return None rather than error
    let metadata = match handle_metadata(
        deps.branch(),
        env,
        operators_addr,
        task_queue,
        task_id,
        fraction_percent,
    ) {
        Ok(x) => x,
        Err(_) => return Ok(None),
    };

    // Get the operators voting power at time of vote creation
    let power: VotingPowerResponse = deps.querier.query_wasm_smart(
        operators_addr.to_string(),
        &OperatorQueryMsg::VotingPowerAtHeight {
            address: operator.to_string(),
            height: Some(metadata.created_height),
        },
    )?;
    if power.power.is_zero() {
        return Err(VerifierError::Unauthorized);
    }

    Ok(Some((metadata, power.power)))
}

fn handle_metadata(
    deps: DepsMut,
    env: &Env,
    operator_addr: &Addr,
    task_queue: &Addr,
    task_id: TaskId,
    fraction_percent: u32,
) -> Result<TaskMetadata, VerifierError> {
    let tasks_storage = TasksStorage::new("tasks");
    let metadata = tasks_storage.get_tasks(deps.storage, (task_queue, task_id))?;

    match metadata {
        Some(meta) => {
            // Ensure this is not yet expired (or completed)
            match meta.status {
                TaskStatus::Completed => Err(VerifierError::TaskAlreadyCompleted),
                TaskStatus::Expired => Err(VerifierError::TaskExpired),
                TaskStatus::Open if meta.is_expired(env) => Err(VerifierError::TaskExpired),
                _ => Ok(meta.clone()),
            }
        }
        None => {
            // We need to query the info from the task queue
            let task_status: TaskStatusResponse = deps.querier.query_wasm_smart(
                task_queue.to_string(),
                &TaskQueryMsg::TaskStatus { id: task_id },
            )?;
            // Abort early if not still open
            match task_status.status {
                TaskStatus::Completed => Err(VerifierError::TaskAlreadyCompleted),
                TaskStatus::Expired => Err(VerifierError::TaskExpired),
                TaskStatus::Open => {
                    // If we create this, we need to calculate total vote power needed
                    let total_power: TotalPowerResponse = deps.querier.query_wasm_smart(
                        operator_addr.to_string(),
                        &OperatorQueryMsg::TotalPowerAtHeight {
                            height: Some(task_status.created_height),
                        },
                    )?;
                    // need to round up!
                    // NOTE: for now we are using percentage, with oracle-verifier
                    // this would change
                    let fraction = Decimal::percent(fraction_percent as u64);
                    let power_required = total_power.power.mul_ceil(fraction);
                    let meta = TaskMetadata {
                        power_required,
                        status: TaskStatus::Open,
                        created_height: task_status.created_height,
                        expires_time: task_status.expires_time,
                    };
                    tasks_storage.save_tasks(deps.storage, (task_queue, task_id), meta.clone())?;
                    Ok(meta)
                }
            }
        }
    }
}
