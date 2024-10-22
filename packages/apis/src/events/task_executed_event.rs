use crate::id::TaskId;
use cosmwasm_std::{Attribute, Event, StdError};

use super::traits::TypedEvent;

#[derive(Debug, Clone, PartialEq)]
pub struct TaskExecutedEvent {
    pub task_id: TaskId,
    pub task_queue: String,
    pub operator: String,
    pub completed: bool,
}

impl TypedEvent for TaskExecutedEvent {
    const NAME: &'static str = "task_executed";
}

impl TryFrom<&Event> for TaskExecutedEvent {
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
                    task_queue = Some(value.to_string());
                }
                "operator" => {
                    operator = Some(value.to_string());
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

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::Event;
    use std::convert::TryFrom;

    #[test]
    fn task_executed_event_simple_parsing() {
        let og_event = TaskExecutedEvent {
            task_id: TaskId::new(7),
            task_queue: "queue_address".to_string(),
            operator: "operator_address".to_string(),
            completed: true,
        };

        let cosm_event: Event = og_event.clone().into();

        let parsed_event = TaskExecutedEvent::try_from(&cosm_event).expect("failed to parse event");

        assert_eq!(og_event, parsed_event);
    }

    #[test]
    fn task_executed_event_with_missing_attribute() {
        let cosm_event = Event::new(TaskExecutedEvent::NAME).add_attributes([
            ("task_id", "7"),
            ("task_queue", "queue_address"),
            // we skip the operator attribute
            ("completed", "true"),
        ]);

        let result = TaskExecutedEvent::try_from(&cosm_event);

        assert!(result.is_err());
    }

    #[test]
    fn task_executed_event_with_incorrect_attribute_key() {
        let cosm_event = Event::new(TaskExecutedEvent::NAME).add_attributes([
            // the typo
            ("task_idd", "7"),
            ("task_queue", "queue_address"),
            ("operator", "operator_address"),
            ("completed", "true"),
        ]);

        let result = TaskExecutedEvent::try_from(&cosm_event);

        assert!(result.is_err());
    }
}
