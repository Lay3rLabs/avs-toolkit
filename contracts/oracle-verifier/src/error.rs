use cosmwasm_std::{Decimal, StdError};
use cw_utils::PaymentError;
use lavs_helpers::verifier::VerifierError;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    ConversionError(#[from] serde_json::Error),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Invalid percentage {1} for value {0}, must be between and 0.1 and 100")]
    InvalidPercentage(String, Decimal),

    #[error("Thresshold not met")]
    ThresholdNotMet,

    #[error("Zero price submitted")]
    ZeroPrice,

    #[error("Operator tried to vote twice: {0}")]
    OperatorAlreadyVoted(String),

    #[error("Task already completed. Cannot vote on it")]
    TaskAlreadyCompleted,

    #[error("Task expired. Cannot vote on it")]
    TaskExpired,

    #[error("Invalid spread configuration. Slashable: {0}. Allowed: {1}.")]
    InvalidSpread(Decimal, Decimal),

    #[error("{0}")]
    Verifier(#[from] VerifierError),

    #[error("Invalid price provided")]
    InvalidPrice,
}
