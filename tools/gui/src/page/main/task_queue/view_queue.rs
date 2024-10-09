use crate::prelude::*;

pub struct TaskQueueViewQueueUi {}

impl TaskQueueViewQueueUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }

    pub fn render(self: &Arc<Self>) -> Dom {
        html!("div", {
            .class(&*TEXT_SIZE_MD)
            .text("View Queue")
        })
    }
}
