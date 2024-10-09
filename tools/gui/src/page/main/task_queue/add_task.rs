use crate::prelude::*;

pub struct TaskQueueAddTaskUi {}

impl TaskQueueAddTaskUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }

    pub fn render(self: &Arc<Self>) -> Dom {
        html!("div", {
            .class(&*TEXT_SIZE_MD)
            .text("Add task")
        })
    }
}
