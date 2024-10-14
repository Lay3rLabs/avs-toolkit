use cosmwasm_std::{Attribute, Event};
use cosmwasm_std::{Decimal, StdError};
use lavs_apis::id::TaskId;

pub trait TypedEvent: TryFrom<Event> + Into<Event> {
    const NAME: &'static str;
    fn is_type(ty: &str) -> bool {
        Self::NAME == ty || Self::NAME == format!("wasm-{ty}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskEvent {
    Executed {
        task_id: TaskId,
        task_queue: String,
        operator: String,
        completed: bool,
    },
    VoteStored {
        task_id: TaskId,
        task_queue_contract: String,
        new_price: Decimal,
    },
    ThresholdNotMet {
        task_id: TaskId,
        task_queue_contract: String,
    },
}

impl TypedEvent for TaskEvent {
    const NAME: &'static str = "task_executed";
}

impl TryFrom<&Event> for TaskEvent {
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
        let mut task_queue: Option<String> = None;
        let mut operator: Option<String> = None;
        let mut completed: Option<bool> = None;

        for Attribute { key, value } in event.attributes.iter() {
            match key.as_str() {
                "task-id" => {
                    if let Ok(value) = value.parse() {
                        task_id = Some(value);
                    }
                }
                "task-queue" => {
                    if let Ok(value) = value.parse() {
                        task_queue = Some(value);
                    }
                }
                "operator" => {
                    if let Ok(value) = value.parse() {
                        operator = Some(value);
                    }
                }
                "completed" => {
                    if let Ok(value) = value.parse() {
                        completed = Some(value)
                    }
                }
                _ => {}
            }
        }

        match (task_id, task_queue, operator, completed) {
            (Some(task_id), Some(task_queue), Some(operator), Some(completed)) => Ok(Self {
                task_id,
                task_queue,
                operator,
                completed,
            }),
            _ => Err(StdError::generic_err(format!(
                "Could not parse fields for {}",
                Self::NAME,
            ))),
        }
    }
}

impl TryFrom<Event> for TaskExecutedEvent {
    type Error = StdError;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        TaskExecutedEvent::try_from(&event)
    }
}

impl From<TaskExecutedEvent> for Event {
    fn from(value: TaskExecutedEvent) -> Self {
        Self::new(TaskExecutedEvent::NAME).add_attributes([
            ("task-id", value.task_id.to_string()),
            ("task-queue", value.task_queue.to_string()),
            ("operator", value.operator.to_string()),
            ("completed", value.completed.to_string()),
        ])
    }
}
