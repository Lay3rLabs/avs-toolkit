use crate::id::TaskId;
use cosmwasm_std::{Attribute, Event, StdError};

use super::traits::TypedEvent;

#[derive(Debug, Clone, PartialEq)]
pub struct TaskCreatedEvent {
    pub task_id: TaskId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TaskCompletedEvent {
    pub task_id: TaskId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TaskExpiredEvent {
    pub task_id: TaskId,
}

impl TypedEvent for TaskCreatedEvent {
    const NAME: &'static str = "task_created_event";
}

impl TypedEvent for TaskCompletedEvent {
    const NAME: &'static str = "task_completed_event";
}

impl TypedEvent for TaskExpiredEvent {
    const NAME: &'static str = "task_expired_event";
}

impl TryFrom<&Event> for TaskCreatedEvent {
    type Error = StdError;

    fn try_from(event: &Event) -> Result<Self, Self::Error> {
        if !Self::is_type(&event.ty) {
            return Err(StdError::generic_err(format!(
                "expected type was {}, but got {}",
                Self::NAME,
                event.ty
            )));
        }

        let mut task_id: Option<TaskId> = None;

        for Attribute { key, value } in event.attributes.iter() {
            if key.as_str() == "task-id" {
                if let Ok(value) = value.parse() {
                    task_id = Some(value);
                }
            }
        }

        match task_id {
            Some(task_id) => Ok(Self { task_id }),
            _ => Err(StdError::generic_err(format!(
                "Could not parse fields for {}",
                Self::NAME,
            ))),
        }
    }
}

impl TryFrom<Event> for TaskCreatedEvent {
    type Error = StdError;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        TaskCreatedEvent::try_from(&event)
    }
}

impl From<TaskCreatedEvent> for Event {
    fn from(value: TaskCreatedEvent) -> Self {
        let mut event = Event::new(TaskCreatedEvent::NAME);

        event = event.add_attributes(vec![Attribute {
            key: "task-id".to_string(),
            value: value.task_id.to_string(),
        }]);

        event
    }
}

impl TryFrom<&Event> for TaskCompletedEvent {
    type Error = StdError;

    fn try_from(event: &Event) -> Result<Self, Self::Error> {
        if !Self::is_type(&event.ty) {
            return Err(StdError::generic_err(format!(
                "expected type was {}, but got {}",
                Self::NAME,
                event.ty
            )));
        }

        let mut task_id: Option<TaskId> = None;

        for Attribute { key, value } in event.attributes.iter() {
            if key.as_str() == "task-id" {
                if let Ok(value) = value.parse() {
                    task_id = Some(value);
                }
            }
        }

        match task_id {
            Some(task_id) => Ok(Self { task_id }),
            _ => Err(StdError::generic_err(format!(
                "Could not parse fields for {}",
                Self::NAME,
            ))),
        }
    }
}

impl TryFrom<Event> for TaskCompletedEvent {
    type Error = StdError;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        TaskCompletedEvent::try_from(&event)
    }
}

impl From<TaskCompletedEvent> for Event {
    fn from(value: TaskCompletedEvent) -> Self {
        let mut event = Event::new(TaskCompletedEvent::NAME);

        event = event.add_attributes(vec![Attribute {
            key: "task-id".to_string(),
            value: value.task_id.to_string(),
        }]);

        event
    }
}

impl TryFrom<&Event> for TaskExpiredEvent {
    type Error = StdError;

    fn try_from(event: &Event) -> Result<Self, Self::Error> {
        if !Self::is_type(&event.ty) {
            return Err(StdError::generic_err(format!(
                "expected type was {}, but got {}",
                Self::NAME,
                event.ty
            )));
        }

        let mut task_id: Option<TaskId> = None;

        for Attribute { key, value } in event.attributes.iter() {
            if key.as_str() == "task-id" {
                if let Ok(value) = value.parse() {
                    task_id = Some(value);
                }
            }
        }

        match task_id {
            Some(task_id) => Ok(Self { task_id }),
            _ => Err(StdError::generic_err(format!(
                "Could not parse fields for {}",
                Self::NAME,
            ))),
        }
    }
}

impl TryFrom<Event> for TaskExpiredEvent {
    type Error = StdError;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        TaskExpiredEvent::try_from(&event)
    }
}

impl From<TaskExpiredEvent> for Event {
    fn from(value: TaskExpiredEvent) -> Self {
        let mut event = Event::new(TaskExpiredEvent::NAME);

        event = event.add_attributes(vec![Attribute {
            key: "task-id".to_string(),
            value: value.task_id.to_string(),
        }]);

        event
    }
}
