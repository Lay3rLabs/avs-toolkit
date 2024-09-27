use cosmwasm_std::{Decimal, Uint128};
use lavs_apis::{
    interfaces::voting::TotalPowerResponse,
    tasks::{TaskStatus, TaskStatusResponse},
    verifier_simple::{OperatorVote, TaskMetadata},
};

pub fn check_vote_validity(
    vote: Option<OperatorVote>,
    voting_power: Uint128,
) -> Result<(), String> {
    if vote.is_some() {
        return Err("Operator already voted".to_string());
    }

    if voting_power.is_zero() {
        return Err("Unauthorized: voting power is zero".to_string());
    }

    Ok(())
}

pub fn initialize_task_metadata(
    task_status: TaskStatusResponse,
    total_power: TotalPowerResponse,
    required_percentage: u32,
) -> Result<TaskMetadata, String> {
    // Abort early if not still open
    match task_status.status {
        TaskStatus::Completed => Err("Task is already completed".to_string()),
        TaskStatus::Expired => Err("Task is expired".to_string()),
        TaskStatus::Open => {
            let fraction = Decimal::percent(required_percentage as u64);
            let power_required = total_power.power.mul_ceil(fraction);
            Ok(TaskMetadata {
                power_required,
                status: TaskStatus::Open,
                created_height: task_status.created_height,
                expires_time: task_status.expires_time,
            })
        }
    }
}
