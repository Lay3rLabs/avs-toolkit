use crate::{prelude::*, util::signal::enumerate_signal};

pub struct EnvsUi {
    envs: MutableVec<EnvData>,
}

#[derive(Clone)]
struct EnvData {
    key: Mutable<Option<String>>,
    value: Mutable<Option<String>>,
    error: Mutable<Option<String>>,
}

impl EnvData {
    fn evaluate(&self) {
        let has_key = self.key.lock_ref().is_some();
        let has_value = self.value.lock_ref().is_some();

        self.error.set(match (has_key, has_value) {
            (true, true) => None,
            (false, false) => Some("Key and Value are required".to_string()),
            (false, _) => Some("Key is required".to_string()),
            (_, false) => Some("Value is required".to_string()),
        });
    }
}

impl EnvsUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            envs: MutableVec::new(),
        })
    }

    pub fn valid_signal(self: &Arc<Self>) -> impl Signal<Item = bool> {
        self.envs
            .signal_vec_cloned()
            .map_signal(|data| {
                map_ref! {
                    let key = data.key.signal_ref(|key| key.is_some()),
                    let value = data.value.signal_ref(|value| value.is_some()),
                    => {
                        *key && *value
                    }
                }
            })
            .to_signal_map(|valids| valids.iter().all(|valid| *valid))
    }

    pub fn extract(self: &Arc<Self>) -> Result<Vec<(String, String)>> {
        let state = self;

        self.envs
            .lock_ref()
            .iter()
            .map(|env| {
                let key = env.key.get_cloned().context("key is required")?;
                let value = env.value.get_cloned().context("value is required")?;
                Ok((key, value))
            })
            .collect()
    }

    pub fn render(self: &Arc<Self>) -> Dom {
        let state = self;

        static SECTIONS: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("gap", "1rem")
            }
        });
        static ROW: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("align-items", "center")
                .style("gap", "1rem")
            }
        });
        Label::new()
            .with_text("Environment Variables")
            .with_direction(LabelDirection::Column)
            .render(html!("div", {
                .class(&*SECTIONS)
                .child(Button::new()
                    .with_text("Add")
                    .with_on_click(clone!(state => move || {
                        let data = EnvData {
                            key: Mutable::new(None),
                            value: Mutable::new(None),
                            error: Mutable::new(None),
                        };
                        data.evaluate();
                        state.envs.lock_mut().push_cloned(data);
                    }))
                    .render()
                )
                .children_signal_vec(enumerate_signal(state.envs.signal_vec_cloned()).map(clone!(state => move |(data, index)| {
                    html!("div", {
                        .class(&*ROW)
                        .children(&mut [
                            Button::new()
                                .with_size(ButtonSize::Sm)
                                .with_color(ButtonColor::Branded)
                                .with_text("Delete")
                                .with_on_click(clone!(state => move || {
                                    state.envs.lock_mut().remove(index);
                                }))
                                .render(),
                            TextInput::new()
                                .with_placeholder("Key")
                                .with_on_input(clone!(data => move |input| {
                                    data.key.set(input);
                                    data.evaluate();
                                }))
                                .render(),
                            TextInput::new()
                                .with_placeholder("Value")
                                .with_on_input(clone!(data => move |input| {
                                    data.value.set(input);
                                    data.evaluate();
                                }))
                                .render(),
                        ])
                        .child_signal(data.error.signal_cloned().map(|error| {
                            match error {
                                Some(error) => Some(html!("div", {
                                    .class([FontSize::Body.class(), &*COLOR_TEXT_INTERACTIVE_ERROR])
                                    .text(&error)
                                })),
                                None => None
                            }
                        }))
                    })
                })))
            }))
    }
}
