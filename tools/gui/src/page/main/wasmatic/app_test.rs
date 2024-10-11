// this file isn't named `test_app` because `test_` prefix causes issues, in VSCode at least
use crate::prelude::*;

pub struct WasmaticTestAppUi {}

impl WasmaticTestAppUi {
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
                    .text("TODO: Wasmatic Test")
                }),
            ])
        })
    }
}
