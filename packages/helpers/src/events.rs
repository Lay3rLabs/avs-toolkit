use cosmwasm_std::Event;
use cosmwasm_std::{Addr, StdError};
use lavs_apis::id::TaskId;

pub trait TypedEvent: TryFrom<Event> + Into<Event> {
    const NAME: &'static str;
}

pub struct TaskExecutedEvent {
    pub task_id: TaskId,
    pub task_queue: Addr,
    pub operator: Addr,
}

impl TypedEvent for TaskExecutedEvent {
    const NAME: &'static str = "task_executed";
}

impl TryFrom<Event> for TaskExecutedEvent {
    type Error = StdError;

    fn try_from(value: Event) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<TaskExecutedEvent> for Event {
    fn from(value: TaskExecutedEvent) -> Self {
        todo!()
    }
}
