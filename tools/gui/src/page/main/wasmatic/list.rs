use std::collections::{hash_map::Entry, HashMap};

use avs_toolkit_shared::wasmatic::{self, AppInfo, AppResponse};
use wasm_bindgen_futures::spawn_local;

use crate::{prelude::*, util::signal::enumerate_signal};

pub struct WasmaticListUi {
    apps: Mutable<Option<MutableVec<Arc<AppEntry>>>>,
    error: Mutable<Option<String>>,
}

impl WasmaticListUi {
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
            }
        });

        html!("div", {
            .child(html!("div", {
                .class(&*TEXT_SIZE_LG)
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
                        .class([&*TEXT_SIZE_LG, &*Color::Red.class()])
                        .text(&error)
                    }))
                }
            })))
            .child_signal(state.apps.signal_cloned().map(clone!(state => move |apps| {
                match apps {
                    None => Some(html!("div", {
                        .class([&*TEXT_SIZE_LG])
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
                                            .class([&*TEXT_SIZE_LG, &*TEXT_WEIGHT_BOLD, &*Color::Accent.class()])
                                            .text(&app.app.name)
                                        }))
                                        .child(Button::new()
                                            .with_color(ButtonColor::Red)
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
                                            .class(&*TEXT_SIZE_LG)
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

struct AppEntry {
    pub app: AppInfo,
    pub endpoints: Vec<String>,
}

async fn get_apps() -> Result<Vec<Arc<AppEntry>>> {
    let mut apps: HashMap<String, AppEntry> = HashMap::new();

    let mut responses = wasmatic::all_apps(
        http_client(),
        CONFIG.chain_info().unwrap_ext().wasmatic.endpoints.clone(),
        |apps| {
            log::info!("apps: {:?}", apps);
        },
    )
    .await?;

    for response in responses.drain(..) {
        for app in response.app.apps {
            match apps.entry(app.name.clone()) {
                Entry::Occupied(mut entry) => {
                    let entry: &mut AppEntry = entry.get_mut();
                    if entry.app != app {
                        return Err(anyhow!("App with the same name but different data"));
                    }
                    entry.endpoints.push(response.endpoint.clone());
                }
                Entry::Vacant(entry) => {
                    entry.insert(AppEntry {
                        app,
                        endpoints: vec![response.endpoint.clone()],
                    });
                }
            }
        }
    }

    Ok(apps.into_iter().map(|(_, v)| Arc::new(v)).collect())
}
