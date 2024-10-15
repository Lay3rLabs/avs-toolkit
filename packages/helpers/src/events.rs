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

#[derive(Debug, Clone, PartialEq)]
pub struct TaskExecutedEvent {
    pub task_id: TaskId,
    pub task_queue: String,
    pub operator: Option<String>,
    pub completed: Option<bool>,
    pub method: Option<String>,
    pub status: Option<String>,
    pub new_price: Option<Decimal>,
    pub action: Option<String>,
}

#[derive(Default)]
pub struct TaskExecutedEventBuilder {
    // Fields corresponding to TaskExecutedEvent
    task_id: Option<TaskId>,
    task_queue: Option<String>,
    operator: Option<String>,
    completed: Option<bool>,
    method: Option<String>,
    status: Option<String>,
    new_price: Option<Decimal>,
    action: Option<String>,
}

impl TaskExecutedEventBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn task_id(mut self, task_id: TaskId) -> Self {
        self.task_id = Some(task_id);
        self
    }

    pub fn task_queue(mut self, task_queue: String) -> Self {
        self.task_queue = Some(task_queue);
        self
    }

    pub fn operator(mut self, operator: String) -> Self {
        self.operator = Some(operator);
        self
    }

    pub fn completed(mut self, completed: bool) -> Self {
        self.completed = Some(completed);
        self
    }

    pub fn method(mut self, method: String) -> Self {
        self.method = Some(method);
        self
    }

    pub fn status(mut self, status: String) -> Self {
        self.status = Some(status);
        self
    }

    pub fn new_price(mut self, new_price: Decimal) -> Self {
        self.new_price = Some(new_price);
        self
    }

    pub fn action(mut self, action: String) -> Self {
        self.action = Some(action);
        self
    }

    pub fn build(self) -> Result<TaskExecutedEvent, StdError> {
        Ok(TaskExecutedEvent {
            task_id: self
                .task_id
                .ok_or_else(|| StdError::generic_err("task_id is required"))?,
            task_queue: self
                .task_queue
                .ok_or_else(|| StdError::generic_err("task_queue is required"))?,
            operator: self.operator,
            completed: self.completed,
            method: self.method,
            status: self.status,
            new_price: self.new_price,
            action: self.action,
        })
    }
}

impl TypedEvent for TaskExecutedEvent {
    const NAME: &'static str = "task_executed";
}

impl TryFrom<&Event> for TaskExecutedEvent {
    type Error = StdError;

    fn try_from(event: &Event) -> Result<Self, Self::Error> {
        if Self::is_type(&event.ty) {
            return Err(StdError::generic_err(format!(
                "expected type was {}, but got {}",
                Self::NAME,
                event.ty
            )));
        }

        let mut builder = TaskExecutedEventBuilder::new();

        for Attribute { key, value } in event.attributes.iter() {
            match key.as_str() {
                "task-id" => {
                    builder = builder.task_id(value.parse().map_err(|_| {
                        StdError::generic_err(format!("Invalid value for task-id: {}", value))
                    })?);
                }
                "task-queue" => {
                    builder = builder.task_queue(value.clone());
                }
                "operator" => {
                    builder = builder.operator(value.clone());
                }
                "completed" => {
                    builder = builder.completed(value.parse().map_err(|_| {
                        StdError::generic_err(format!("Invalid value for completed: {}", value))
                    })?);
                }
                "method" => {
                    builder = builder.method(value.clone());
                }
                "status" => {
                    builder = builder.status(value.clone());
                }
                "new_price" => {
                    builder = builder.new_price(value.parse().map_err(|_| {
                        StdError::generic_err(format!("Invalid value for new_price: {}", value))
                    })?);
                }
                "action" => {
                    builder = builder.action(value.clone());
                }
                _ => {}
            }
        }

        builder.build()
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
        let mut event = Event::new(TaskExecutedEvent::NAME);

        event = event.add_attributes(vec![
            Attribute {
                key: "task-id".to_string(),
                value: value.task_id.to_string(),
            },
            Attribute {
                key: "task-queue".to_string(),
                value: value.task_queue,
            },
        ]);

        if let Some(operator) = value.operator {
            event = event.add_attribute("operator", operator);
        }
        if let Some(completed) = value.completed {
            event = event.add_attribute("completed", completed.to_string());
        }
        if let Some(method) = value.method {
            event = event.add_attribute("method", method);
        }
        if let Some(status) = value.status {
            event = event.add_attribute("status", status);
        }
        if let Some(new_price) = value.new_price {
            event = event.add_attribute("new_price", new_price.to_string());
        }
        if let Some(action) = value.action {
            event = event.add_attribute("action", action);
        }

        event
    }
}
