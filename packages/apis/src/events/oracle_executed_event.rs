use std::{fmt, str::FromStr};

use crate::id::TaskId;
use cosmwasm_std::{Attribute, Decimal, Event, StdError, StdResult};

use super::traits::TypedEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum OracleExecutionStatus {
    VoteStored,
    ThresholdMet,
    ThresholdNotMet,
}

impl fmt::Display for OracleExecutionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_str = match self {
            OracleExecutionStatus::VoteStored => "vote_stored",
            OracleExecutionStatus::ThresholdMet => "threshold_met",
            OracleExecutionStatus::ThresholdNotMet => "threshold_not_met",
        };
        write!(f, "{}", status_str)
    }
}

impl FromStr for OracleExecutionStatus {
    type Err = StdError;

    fn from_str(s: &str) -> StdResult<Self> {
        match s {
            "vote_stored" => Ok(OracleExecutionStatus::VoteStored),
            "threshold_met" => Ok(OracleExecutionStatus::ThresholdMet),
            "threshold_not_met" => Ok(OracleExecutionStatus::ThresholdNotMet),
            _ => Err(StdError::generic_err(format!("Invalid status: {}", s))),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OracleExecutedEvent {
    pub task_id: TaskId,
    pub status: OracleExecutionStatus,
    pub new_price: Option<Decimal>,
    pub task_queue_contract: String,
}

impl TypedEvent for OracleExecutedEvent {
    const NAME: &'static str = "oracle_executed_event";
}

impl TryFrom<&Event> for OracleExecutedEvent {
    type Error = StdError;

    fn try_from(event: &Event) -> Result<Self, Self::Error> {
        if Self::is_type(&event.ty) {
            return Err(StdError::generic_err(format!(
                "expected type was {}, but got {}",
                Self::NAME,
                event.ty
            )));
        }

        let mut task_id: Option<TaskId> = None;
        let mut status: Option<OracleExecutionStatus> = None;
        let mut new_price: Option<Decimal> = None;
        let mut task_queue_contract: Option<String> = None;

        for Attribute { key, value } in event.attributes.iter() {
            match key.as_str() {
                "task-id" => {
                    if let Ok(value) = value.parse() {
                        task_id = Some(value);
                    }
                }
                "status" => {
                    if let Ok(value) = value.parse() {
                        status = Some(value);
                    }
                }
                "new-price" => {
                    if let Ok(value) = value.parse() {
                        new_price = Some(value);
                    }
                }
                "task-queue-contract" => {
                    task_queue_contract = Some(value.clone());
                }
                _ => {}
            }
        }

        match (task_id, status, new_price, task_queue_contract) {
            (Some(task_id), Some(status), Some(new_price), Some(task_queue_contract)) => Ok(Self {
                task_id,
                status,
                new_price: Some(new_price),
                task_queue_contract,
            }),
            _ => Err(StdError::generic_err(format!(
                "Could not parse fields for {}",
                Self::NAME,
            ))),
        }
    }
}

impl TryFrom<Event> for OracleExecutedEvent {
    type Error = StdError;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        OracleExecutedEvent::try_from(&event)
    }
}

impl From<OracleExecutedEvent> for Event {
    fn from(value: OracleExecutedEvent) -> Self {
        let mut event = Event::new(OracleExecutedEvent::NAME);

        let mut attributes = vec![
            Attribute {
                key: "task-id".to_string(),
                value: value.task_id.to_string(),
            },
            Attribute {
                key: "status".to_string(),
                value: value.status.to_string(),
            },
            Attribute {
                key: "task-queue-contract".to_string(),
                value: value.task_queue_contract,
            },
        ];

        if let Some(new_price) = value.new_price {
            attributes.push(Attribute {
                key: "new-price".to_string(),
                value: new_price.to_string(),
            });
        }

        event = event.add_attributes(attributes);

        event
    }
}
