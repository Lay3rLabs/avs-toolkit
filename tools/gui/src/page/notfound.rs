use crate::prelude::*;

pub struct NotFoundUi {}

impl NotFoundUi {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self) -> Dom {
        html!("div", {
            .text("not found!")
        })
    }
}
