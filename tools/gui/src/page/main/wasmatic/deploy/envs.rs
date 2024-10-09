use crate::prelude::*;

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

        Label::new()
            .with_text("Environment Variables")
            .with_direction(LabelDirection::Column)
            .render(html!("div", {
                .children_signal_vec(state.envs.signal_vec_cloned().map(clone!(state => move |data| {
                    html!("div", {
                        .style("display", "flex")
                        .style("gap", "1rem")
                        .children(&mut [
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
                                    .class([&*TEXT_SIZE_SM, Color::Red.class()])
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
