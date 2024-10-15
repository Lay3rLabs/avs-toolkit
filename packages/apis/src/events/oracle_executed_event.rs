use crate::id::TaskId;
use cosmwasm_std::{Attribute, Decimal, Event, StdError};

use super::traits::TypedEvent;

#[derive(Debug, Clone, PartialEq)]
pub struct OracleExecutedEvent {
    pub task_id: TaskId,
    pub method: String,
    pub status: String,
    pub new_price: Decimal,
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
        let mut method: Option<String> = None;
        let mut status: Option<String> = None;
        let mut new_price: Option<Decimal> = None;
        let mut task_queue_contract: Option<String> = None;

        for Attribute { key, value } in event.attributes.iter() {
            match key.as_str() {
                "task-id" => {
                    if let Ok(value) = value.parse() {
                        task_id = Some(value);
                    }
                }
                "method" => {
                    method = Some(value.clone());
                }
                "status" => {
                    status = Some(value.clone());
                }
                "new_price" => {
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

        match (task_id, method, status, new_price, task_queue_contract) {
            (
                Some(task_id),
                Some(method),
                Some(status),
                Some(new_price),
                Some(task_queue_contract),
            ) => Ok(Self {
                task_id,
                method,
                status,
                new_price,
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

        event = event.add_attributes(vec![
            Attribute {
                key: "task-id".to_string(),
                value: value.task_id.to_string(),
            },
            Attribute {
                key: "method".to_string(),
                value: value.method,
            },
            Attribute {
                key: "status".to_string(),
                value: value.status,
            },
            Attribute {
                key: "new_price".to_string(),
                value: value.new_price.to_string(),
            },
            Attribute {
                key: "task-queue-contract".to_string(),
                value: value.task_queue_contract,
            },
        ]);

        event
    }
}
