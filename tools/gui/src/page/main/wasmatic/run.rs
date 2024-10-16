use crate::prelude::*;

pub struct WasmaticRunUi {}

impl WasmaticRunUi {
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
            .class([&*CONTAINER, FontSize::Header.class()])
            .children(&mut [
                html!("div", {
                    .text("TODO: Wasmatic Run")
                }),
            ])
        })
    }
}
