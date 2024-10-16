use crate::prelude::*;

pub struct PermissionsUi {
    permissions: Mutable<Option<serde_json::Value>>,
    error: Mutable<Option<String>>,
}

impl PermissionsUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            permissions: Mutable::new(None),
            error: Mutable::new(None),
        })
    }

    pub fn valid_signal(self: &Arc<Self>) -> impl Signal<Item = bool> {
        self.error.signal_ref(|error| error.is_none())
    }

    pub fn extract(self: &Arc<Self>) -> Option<serde_json::Value> {
        self.permissions.get_cloned()
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
            .class(&*CONTAINER)
            .child(Label::new()
                .with_text("Permissions (in JSON format)")
                .with_direction(LabelDirection::Column)
                .render(TextArea::new()
                    .with_mixin(|dom| {
                        dom
                            .style("width", "30rem")
                            .style("height", "5rem")
                    })
                    .with_on_input(clone!(state => move |input| {
                        match input {
                            None => {
                                state.permissions.set(None);
                                state.error.set_neq(None);
                            },
                            Some(value) => {
                                match serde_json::from_str(&value) {
                                    Ok(value) => {
                                        state.error.set_neq(None);
                                        state.permissions.set(Some(value));
                                    },
                                    Err(err) => {
                                        state.error.set_neq(Some(err.to_string()));
                                    }
                                }
                            }
                        }
                    }))
                    .render()
                )
            )
            .child_signal(state.error.signal_cloned().map(|error| {
                match error {
                    Some(error) => {
                        Some(html!("div", {
                            .class([FontSize::Body.class(), &*COLOR_TEXT_INTERACTIVE_ERROR])
                            .text(&error)
                        }))
                    },
                    None => None
                }
            }))
        })
    }
}
