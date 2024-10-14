use cosmwasm_std::StdError;
use cosmwasm_std::{Attribute, Event};
use lavs_apis::id::TaskId;

pub trait TypedEvent: TryFrom<Event> + Into<Event> {
    const NAME: &'static str;
}

pub struct TaskExecutedEvent {
    pub task_id: TaskId,
    pub task_queue: String,
    pub operator: String,
}

impl TypedEvent for TaskExecutedEvent {
    const NAME: &'static str = "task_executed";
}

impl TryFrom<Event> for TaskExecutedEvent {
    type Error = StdError;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if event.ty != Self::NAME {
            return Err(StdError::generic_err(format!(
                "expected type was {}, but got {}",
                Self::NAME,
                event.ty
            )));
        }

        let mut task_id: Option<TaskId> = None;
        let mut task_queue: Option<String> = None;
        let mut operator: Option<String> = None;

        for Attribute { key, value } in event.attributes {
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
                _ => {}
            }
        }

        match (task_id, task_queue, operator) {
            (Some(task_id), Some(task_queue), Some(operator)) => Ok(Self {
                task_id,
                task_queue,
                operator,
            }),
            _ => Err(StdError::generic_err(format!(
                "Could not parse fields for {}",
                Self::NAME,
            ))),
        }
    }
}

impl From<TaskExecutedEvent> for Event {
    fn from(value: TaskExecutedEvent) -> Self {
        Self::new(TaskExecutedEvent::NAME).add_attributes([
            ("task-id", value.task_id.to_string()),
            ("task-queue", value.task_queue.to_string()),
            ("operator", value.operator.to_string()),
        ])
    }
}
