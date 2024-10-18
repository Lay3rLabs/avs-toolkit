use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use lavs_apis::verifier_simple::VerifierError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Verifier(#[from] VerifierError),

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
