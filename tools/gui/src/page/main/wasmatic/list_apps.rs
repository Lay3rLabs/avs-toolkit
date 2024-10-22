use std::collections::{hash_map::Entry, HashMap};

use avs_toolkit_shared::wasmatic::{self, AppInfo, AppResponse};
use wasm_bindgen_futures::spawn_local;

use crate::{page::main::wasmatic::get_apps, prelude::*, util::signal::enumerate_signal};

use super::AppEntry;

pub struct WasmaticListAppsUi {
    apps: Mutable<Option<MutableVec<Arc<AppEntry>>>>,
    error: Mutable<Option<String>>,
}

impl WasmaticListAppsUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            apps: Mutable::new(None),
            error: Mutable::new(None),
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

        static ROW: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("align-items", "flex-start")
                .style("gap", "1rem")
            }
        });

        static APP: LazyLock<String> = LazyLock::new(|| {
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
            .child(html!("div", {
                .class(FontSize::Header.class())
                .style("margin-bottom", "1rem")
                .text("Current Wasmatic Apps")
            }))
            .future(clone!(state => async move {
                match get_apps().await {
                    Ok(apps) => {
                        state.apps.set(Some(MutableVec::new_with_values(apps)));
                    },
                    Err(err) => {
                        state.error.set(Some(err.to_string()));
                    }
                }
            }))
            .child_signal(state.error.signal_cloned().map(clone!(state => move |error| {
                match error {
                    None => None,
                    Some(error) => Some(html!("div", {
                        .class([FontSize::Header.class(), &COLOR_TEXT_INTERACTIVE_ERROR])
                        .text(&error)
                    }))
                }
            })))
            .child_signal(state.apps.signal_cloned().map(clone!(state => move |apps| {
                match apps {
                    None => Some(html!("div", {
                        .class([FontSize::Header.class()])
                        .text("Loading...")
                    })),
                    Some(apps) => Some(html!("div", {
                        .class([&*CONTAINER])
                        .children_signal_vec(enumerate_signal(apps.signal_vec_cloned()).map(clone!(state => move |(app, index)| {
                            html!("div", {
                                .class(&*APP)
                                .children([
                                    html!("div", {
                                        .class(&*ROW)
                                        .child(html!("div", {
                                            .class([FontSize::Header.class(), FontWeight::Bold.class(), &*ColorText::Brand.color_class()])
                                            .text(&app.app.name)
                                        }))
                                        .child(Button::new()
                                            .with_color(ButtonColor::Branded)
                                            .with_size(ButtonSize::Sm)
                                            .with_text("Remove")
                                            .with_on_click(clone!(state, app => move || {
                                                state.error.set(None);
                                                spawn_local(clone!(state, app => async move {
                                                    match wasmatic::remove(
                                                        http_client(),
                                                        app.endpoints.clone(),
                                                        app.app.name.clone(),
                                                        |response| {
                                                            log::info!("response: {:?}", response);
                                                        }
                                                    ).await {
                                                        Err(err) => {
                                                            state.error.set(Some(err.to_string()));
                                                        },
                                                        Ok(_) => {
                                                            state.apps.lock_ref().as_ref().unwrap_ext().lock_mut().remove(index);
                                                        }
                                                    }
                                                }))
                                            }))
                                            .render())
                                    }),
                                    {
                                        let endpoints = &app.endpoints;
                                        let app = &app.app;

                                        html!("ul", {
                                            .class(FontSize::Header.class())
                                            .children([
                                                html!("li", {
                                                    .text(&format!("digest: {:?}", app.digest))
                                                }),
                                                html!("li", {
                                                    .text(&format!("testable: {:?}", app.testable))
                                                }),
                                                html!("li", {
                                                    .text(&format!("endpoints: {:?}", endpoints))
                                                }),
                                                html!("li", {
                                                    .text(&format!("permissions: {:?}", serde_json::to_string(&app.permissions).unwrap_or_else(|_| "error!".to_string())))
                                                }),
                                            ])
                                            .apply(|dom| {
                                                match &app.trigger {
                                                    wasmatic::Trigger::Cron { schedule } => {
                                                        dom.child(html!("li", {
                                                            .text("Trigger: CRON")
                                                            .child(html!("ul", {
                                                                .child(html!("li", {
                                                                    .text(&format!("schedule: {:?}", schedule))
                                                                }))
                                                            }))
                                                        }))
                                                    },
                                                    wasmatic::Trigger::Queue {task_queue_addr, hd_index, poll_interval} => {
                                                        dom.child(html!("li", {
                                                            .text("Trigger: TASK QUEUE")
                                                            .child(html!("ul", {
                                                                .children([
                                                                    html!("li", {
                                                                        .text(&format!("address: {}", task_queue_addr))
                                                                    }),
                                                                    html!("li", {
                                                                        .text(&format!("interval: {} seconds", poll_interval))
                                                                    }),
                                                                    html!("li", {
                                                                        .text(&format!("hd index: {}", hd_index))
                                                                    }),
                                                                ])
                                                            }))
                                                        }))
                                                    }
                                                }
                                            })
                                        })
                                    }
                                ])
                            })
                        })))
                    }))
                }
            })))
        })
    }
}
