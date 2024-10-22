use crate::prelude::*;
use dominator_helpers::futures::AsyncLoader;
use layer_climb::proto::abci::TxResponse;

pub struct ContractInstantiateUi {
    pub loader: AsyncLoader,
    pub code_id: Mutable<Option<u64>>,
    pub msg: Mutable<Option<String>>,
    pub error: Mutable<Option<String>>,
    pub success: Mutable<Option<(Address, TxResponse)>>,
}

impl ContractInstantiateUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            loader: AsyncLoader::new(),
            code_id: Mutable::new(None),
            msg: Mutable::new(None),
            error: Mutable::new(None),
            success: Mutable::new(None),
        })
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
                .with_text("Code ID")
                .with_direction(LabelDirection::Column)
                .render(TextInput::new()
                    .with_kind(TextInputKind::Number)
                    .with_placeholder("e.g. 123")
                    .with_on_input(clone!(state => move |code_id| {
                        match code_id {
                            None => state.code_id.set(None),
                            Some(code_id) => {
                                let code_id = code_id.parse::<u64>().ok();
                                state.code_id.set(code_id);
                            }
                        }
                    }))
                    .render()
                )
            )
            .child(Label::new()
                .with_text("Message (optional)")
                .with_direction(LabelDirection::Column)
                .render(
                    TextArea::new()
                    .with_placeholder(r#"e.g. {\"foo\":\"bar\"}"#)
                    .with_mixin(|dom| {
                        dom
                            .style("width", "30rem")
                            .style("height", "10rem")
                    })
                    .with_on_input(clone!(state => move |msg| {
                        state.msg.set(msg);
                    }))
                    .render()
                )
            )
            .child(html!("div", {
                .child(Button::new()
                    .with_text("Instantiate")
                    .with_disabled_signal(state.validate_signal().map(|valid| !valid))
                    .with_on_click(clone!(state => move || {
                        state.loader.load(clone!(state => async move {
                            state.error.set(None);
                            state.success.set(None);
                            let code_id = state.code_id.get_cloned().unwrap_ext();
                            let msg = state.msg.get_cloned();
                            match contract_str_to_msg(msg.as_deref()) {
                                Err(err) => {
                                    state.error.set(Some(err.to_string()));
                                },
                                Ok(msg) => {
                                    let client = signing_client();
                                    let resp = client.contract_instantiate(
                                        client.addr.clone(),
                                        code_id,
                                        "instantiate".to_string(),
                                        &msg,
                                        Vec::new(),
                                        None,
                                    ).await;

                                    match resp {
                                        Ok(resp) => {
                                            state.success.set(Some(resp));
                                        },
                                        Err(err) => {
                                            state.error.set(Some(err.to_string()));
                                        }
                                    }
                                }
                            }

                        }));
                    }))
                    .render()
                )
            }))
            .child_signal(state.loader.is_loading().map(|is_loading| {
                match is_loading {
                    true => Some(html!("div", {
                        .class(FontSize::Body.class())
                        .text("Uploading...")
                    })),
                    false => None
                }
            }))
            .child_signal(state.success.signal_cloned().map(|success| {
                match success {
                    Some((addr, tx_resp)) => Some(html!("div", {
                        .child(html!("div", {
                            .class([FontSize::Body.class(), ColorText::Body.color_class()])
                            .text(&format!("Contract instantiated! address: {}", addr))
                        }))
                        .child(html!("div", {
                            .class([FontSize::Body.class(), ColorText::Brand.color_class()])
                            .text(&format!("Tx Hash: {}", tx_resp.txhash))
                        }))
                    })),
                    None => None
                }
            }))
            .child_signal(state.error.signal_cloned().map(|error| {
                match error {
                    Some(error) => Some(html!("div", {
                        .class([FontSize::Body.class(), &*COLOR_TEXT_INTERACTIVE_ERROR])
                        .text(&error)
                    })),
                    None => None
                }
            }))
        })
    }

    fn validate_signal(&self) -> impl Signal<Item = bool> {
        self.code_id
            .signal_cloned()
            .map(|code_id| code_id.is_some())
    }
}
