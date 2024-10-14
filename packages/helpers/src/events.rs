use cosmwasm_std::{Event as CwEvent, StdError};
use cosmwasm_std::{Event, StdResult};

pub trait TypedEvent: Sized {
    const NAME: &'static str;

    fn into_event(self) -> Event;
    fn try_from_event(event: Event) -> StdResult<Self>;

    fn event_name() -> &'static str {
        Self::NAME
    }
}

pub struct TaskExecuted {
    pub task_id: String,
    pub task_queue: String,
    pub operator: String,
}

impl Event for TaskExecuted {
    const NAME: &'static str = "task_executed";

    fn to_event(&self) -> CwEvent {
        CwEvent::new(Self::NAME)
            .add_attribute("task_id", &self.task_id)
            .add_attribute("task_queue", &self.task_queue)
            .add_attribute("operator", &self.operator)
    }

    fn from_event(event: CwEvent) -> StdResult<Self> {
        if event.ty != Self::NAME {
            return Err(StdError::generic_err(format!(
                "Expected event type '{}', got '{}'",
                Self::NAME,
                event.ty
            )));
        }

        let mut task_id = None;
        let mut task_queue = None;
        let mut operator = None;

        for attr in event.attributes {
            match attr.key.as_str() {
                "task_id" => task_id = Some(attr.value),
                "task_queue" => task_queue = Some(attr.value),
                "operator" => operator = Some(attr.value),
                _ => {}
            }
        }

        Ok(TaskExecuted {
            task_id: task_id.ok_or_else(|| StdError::not_found("task_id"))?,
            task_queue: task_queue.ok_or_else(|| StdError::not_found("task_queue"))?,
            operator: operator.ok_or_else(|| StdError::not_found("operator"))?,
        })
    }
}
