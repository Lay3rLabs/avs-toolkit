use crate::prelude::*;

pub struct WasmaticRemoveUi {}

impl WasmaticRemoveUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }

    pub fn render(self: &Arc<Self>) -> Dom {
        let state = self;

        static CONTAINER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("gap", "1rem")
            }
        });

        html!("div", {
            .class([&*CONTAINER, &*TEXT_SIZE_LG])
            .children(&mut [
                html!("div", {
                    .text("TODO: Wasmatic Remove")
                }),
            ])
        })
    }
}
