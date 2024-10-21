use crate::{define_task_queue_event, id::TaskId};
use cosmwasm_std::{Attribute, Event, StdError};

use super::traits::TypedEvent;

define_task_queue_event!(TaskCreatedEvent, "task_created_event");
define_task_queue_event!(TaskCompletedEvent, "task_completed_event");
define_task_queue_event!(TaskExpiredEvent, "task_expired_event");

#[macro_export]
macro_rules! define_task_queue_event {
    ($struct_name:ident, $event_name:expr) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $struct_name {
            pub task_id: TaskId,
        }

        impl TypedEvent for $struct_name {
            const NAME: &'static str = $event_name;
        }

        impl TryFrom<&Event> for $struct_name {
            type Error = StdError;

            fn try_from(event: &Event) -> Result<Self, Self::Error> {
                if !Self::is_type(&event.ty) {
                    return Err(StdError::generic_err(format!(
                        "Expected event type '{}', but got '{}'",
                        Self::NAME,
                        event.ty
                    )));
                }

                let mut task_id: Option<TaskId> = None;

                for Attribute { key, value } in event.attributes.iter() {
                    if key.as_str() == "task-id" {
                        task_id = Some(value.parse().map_err(|e| {
                            StdError::generic_err(format!("Failed to parse 'task_id': {}", e))
                        })?);
                    }
                }

                match task_id {
                    Some(task_id) => Ok(Self { task_id }),
                    None => Err(StdError::generic_err(format!(
                        "Could not parse 'task-id' field for '{}'",
                        Self::NAME
                    ))),
                }
            }
        }

        impl TryFrom<Event> for $struct_name {
            type Error = StdError;

            fn try_from(event: Event) -> Result<Self, Self::Error> {
                $struct_name::try_from(&event)
            }
        }

        impl From<$struct_name> for Event {
            fn from(value: $struct_name) -> Self {
                let event = Event::new($struct_name::NAME)
                    .add_attribute("task-id", value.task_id.to_string());
                event
            }
        }
    };
}
