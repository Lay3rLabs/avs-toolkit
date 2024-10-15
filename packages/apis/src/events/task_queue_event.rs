use crate::id::TaskId;
use cosmwasm_std::{Attribute, Event, StdError};

use super::traits::TypedEvent;

#[derive(Debug, Clone, PartialEq)]
pub struct TaskQueueEvent {
    pub task_id: TaskId,
    pub action: String,
}

impl TypedEvent for TaskQueueEvent {
    const NAME: &'static str = "task_queue_event";
}

impl TryFrom<&Event> for TaskQueueEvent {
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
        let mut action: Option<String> = None;

        for Attribute { key, value } in event.attributes.iter() {
            match key.as_str() {
                "task-id" => {
                    if let Ok(value) = value.parse() {
                        task_id = Some(value);
                    }
                }
                "action" => {
                    action = Some(value.clone());
                }
                _ => {}
            }
        }

        match (task_id, action) {
            (Some(task_id), Some(action)) => Ok(Self { task_id, action }),
            _ => Err(StdError::generic_err(format!(
                "Could not parse fields for {}",
                Self::NAME,
            ))),
        }
    }
}

impl TryFrom<Event> for TaskQueueEvent {
    type Error = StdError;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        TaskQueueEvent::try_from(&event)
    }
}

impl From<TaskQueueEvent> for Event {
    fn from(value: TaskQueueEvent) -> Self {
        let mut event = Event::new(TaskQueueEvent::NAME);

        event = event.add_attributes(vec![
            Attribute {
                key: "task-id".to_string(),
                value: value.task_id.to_string(),
            },
            Attribute {
                key: "action".to_string(),
                value: value.action,
            },
        ]);

        event
    }
}
