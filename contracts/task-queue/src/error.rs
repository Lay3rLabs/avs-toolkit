use cosmwasm_std::StdError;
use cw_ownable::OwnershipError;
use cw_utils::PaymentError;
use lavs_apis::{id::TaskId, interfaces::task_hooks::TaskHookError, time::Duration};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    TaskHook(#[from] TaskHookError),

    #[error("{0}")]
    Ownership(#[from] OwnershipError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Timeout Info is invalid")]
    InvalidTimeoutInfo,

    #[error("Timeout is shorter than allowed minimum {0}")]
    TimeoutTooShort(Duration),

    #[error("Timeout is longer than allowed maximum {0}")]
    TimeoutTooLong(Duration),

    #[error("You need to pay at least {0} {1} to create a task")]
    InsufficientPayment(u128, String),

    #[error("Task is completed")]
    TaskCompleted,

    #[error("Task is expired")]
    TaskExpired,

    #[error("Task is not yet expired")]
    TaskNotExpired,

    #[error("Missing result for completed task {id}")]
    MissingResultCompleted { id: TaskId },

    #[error("Unknown reply id {id}")]
    UnknownReplyId { id: u64 },
}
