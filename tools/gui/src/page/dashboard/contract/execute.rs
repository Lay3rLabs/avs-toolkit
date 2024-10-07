use crate::prelude::*;
use dominator_helpers::futures::AsyncLoader;
use layer_climb::proto::abci::TxResponse;

pub struct ContractExecuteUi {
    pub loader: AsyncLoader,
    pub address: Mutable<Option<Address>>,
    pub msg: Mutable<Option<String>>,
    pub error: Mutable<Option<String>>,
    pub success: Mutable<Option<TxResponse>>,
    pub client: SigningClient,
}

impl ContractExecuteUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            loader: AsyncLoader::new(),
            address: Mutable::new(None),
            msg: Mutable::new(None),
            error: Mutable::new(None),
            success: Mutable::new(None),
            client: CLIENT.get().unwrap_ext().clone(),
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
                .with_text("Address")
                .with_direction(LabelDirection::Column)
                .render(TextInput::new()
                    .with_placeholder("e.g. slayaddr...")
                    .with_mixin(|dom| {
                        dom
                            .style("width", "30rem")
                    })
                    .with_on_input(clone!(state => move |address| {
                        match address {
                            None => state.address.set(None),
                            Some(address) => {
                                let address = state.client.querier.chain_config.parse_address(&address).ok();
                                state.address.set(address);
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
                    .with_text("Execute")
                    .with_disabled_signal(state.validate_signal().map(|valid| !valid))
                    .with_on_click(clone!(state => move || {
                        state.loader.load(clone!(state => async move {
                            state.error.set(None);
                            state.success.set(None);
                            let address = state.address.get_cloned().unwrap_ext();
                            let msg = state.msg.get_cloned();
                            match contract_str_to_msg(msg.as_deref()) {
                                Err(err) => {
                                    state.error.set(Some(err.to_string()));
                                },
                                Ok(msg) => {
                                    let resp = state.client.contract_execute(
                                        &address,
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
                        .class(&*TEXT_SIZE_MD)
                        .text("Uploading...")
                    })),
                    false => None
                }
            }))
            .child_signal(state.success.signal_cloned().map(|success| {
                match success {
                    Some(tx_resp) => Some(html!("div", {
                        .child(html!("div", {
                            .class([&*TEXT_SIZE_MD, Color::Darkish.class()])
                            .text(&format!("Contract executed! tx hash: {}", tx_resp.txhash))
                        }))
                    })),
                    None => None
                }
            }))
            .child_signal(state.error.signal_cloned().map(|error| {
                match error {
                    Some(error) => Some(html!("div", {
                        .class([&*TEXT_SIZE_SM, Color::Red.class()])
                        .text(&error)
                    })),
                    None => None
                }
            }))
        })
    }

    fn validate_signal(&self) -> impl Signal<Item = bool> {
        self.address
            .signal_cloned()
            .map(|address| address.is_some())
    }
}
