use avs_toolkit_shared::wasmatic::{self, InfoResponse};

use crate::prelude::*;

pub struct WasmaticInfoUi {
    error: Mutable<Option<String>>,
    infos: Mutable<Option<Vec<Arc<InfoResponse>>>>,
}

impl WasmaticInfoUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            error: Mutable::new(None),
            infos: Mutable::new(None),
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

        static INFO: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("gap", "0.5rem")
                .style("padding", "1rem")
                .style("border-radius", "0.25rem")
                .style("border-width", "1px")
                .style("border-style", "solid")
            }
        });

        html!("div", {
            .future(clone!(state => async move {
                let response = wasmatic::info(
                    http_client(),
                    CONFIG.chain_info().unwrap_ext().wasmatic.endpoints.clone(),
                    |response| {
                        log::info!("Wasmatic Info: {:?}", response);
                    }
                ).await;

                match response {
                    Ok(infos) => {
                        let infos = infos.into_iter().map(Arc::new).collect();
                        state.infos.set(Some(infos));
                    },
                    Err(err) => {
                        state.error.set(Some(err.to_string()));
                    }
                }
            }))
            .class([&*CONTAINER, FontSize::Header.class()])
            .child_signal(state.error.signal_cloned().map(|error| {
                error.map(|error| {
                    html!("div", {
                        .class(&*COLOR_TEXT_INTERACTIVE_ERROR)
                        .text(&error)
                    })
                })
            }))
            .child_signal(state.infos.signal_cloned().map(|infos| {
                infos.map(|infos| {
                    html!("div", {
                        .children(infos.iter().map(|info| {
                            html!("div", {
                                .class(&*INFO)
                                .child(html!("div", {
                                    .text(&format!("Endpoint: {}", info.endpoint))
                                }))
                                .child(html!("div", {
                                    .text("Operators")
                                    .child(html!("ul", {
                                        .children(info.response.operators.iter().map(|operator| {
                                            html!("li", {
                                                .text(operator)
                                            })
                                        }))
                                    }))
                                }))
                            })
                        }))
                    })
                })
            }))
        })
    }
}
