use crate::prelude::*;
use dominator_helpers::futures::AsyncLoader;
use layer_climb::proto::abci::TxResponse;
use wasm_bindgen_futures::JsFuture;
use web_sys::{js_sys, File};

pub struct ContractUploadUi {
    pub loader: AsyncLoader,
    pub file: Mutable<Option<File>>,
    pub error: Mutable<Option<String>>,
    pub success: Mutable<Option<(u64, TxResponse)>>,
}

impl ContractUploadUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            loader: AsyncLoader::new(),
            file: Mutable::new(None),
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
            .child(html!("label", {
                .class(FontSize::Header.class())
                .attr("for", "contract-upload")
                .text("Choose a .wasm file")
            }))
            .child(html!("input" => web_sys::HtmlInputElement, {
                .attrs!{
                    "type": "file",
                    "id": "contract-upload",
                    "accept": ".wasm"
                }
                .with_node!(elem => {
                    .event(clone!(elem, state => move |evt:events::Change| {
                        if let Some(file) = elem.files().and_then(|files| files.item(0)) {
                            state.file.set(Some(file));
                        }
                    }))
                })
            }))
            .child(html!("div", {
                .child(Button::new()
                    .with_text("Upload")
                    .with_disabled_signal(state.file.signal_cloned().map(|file| file.is_none()))
                    .with_on_click(clone!(state => move || {
                        state.loader.load(clone!(state => async move {
                            state.error.set(None);
                            state.success.set(None);
                            let file = state.file.get_cloned().unwrap_ext();
                            match JsFuture::from(file.array_buffer()).await {
                                Ok(array_buffer) => {
                                    let wasm_byte_code = js_sys::Uint8Array::new(&array_buffer).to_vec();
                                    let client = signing_client();
                                    let mut tx_builder = client.tx_builder();
                                    tx_builder.set_gas_simulate_multiplier(2.0);

                                    match client.contract_upload_file(wasm_byte_code, Some(tx_builder)).await {
                                        Ok((code_id, tx_resp)) => {
                                            state.success.set(Some((code_id, tx_resp)));
                                        },
                                        Err(err) => {
                                            log::error!("{:?}", err);
                                            state.error.set(Some(format!("Error uploading contract: {:?}", err)));
                                        }
                                    }
                                },
                                Err(err) => {
                                    state.error.set(Some(format!("Error reading file: {:?}", err)));
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
                    Some((code_id, tx_resp)) => Some(html!("div", {
                        .child(html!("div", {
                            .class([FontSize::Body.class(), ColorText::Body.color_class()])
                            .text(&format!("Contract uploaded! Code ID: {}", code_id))
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
}
