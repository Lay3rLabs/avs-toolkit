use std::collections::{hash_map::Entry, HashMap};

use avs_toolkit_shared::wasmatic::{self, AppInfo, AppResponse};
use wasm_bindgen_futures::spawn_local;

use crate::{prelude::*, util::signal::enumerate_signal};

pub struct WasmaticRemoveUi {
    apps: Mutable<Option<MutableVec<Arc<AppEntry>>>>,
    error: Mutable<Option<String>>,
}

impl WasmaticRemoveUi {
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
                .style("align-items", "center")
                .style("gap", "1rem")
            }
        });

        html!("div", {
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
                                .class([&*ROW])
                                .children([
                                    Button::new()
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
                                        .render(),
                                    html!("div", {
                                        .class(&*TEXT_SIZE_LG)
                                        .text(&app.app.name)
                                    }),
                                    html!("div", {
                                        .class(&*TEXT_SIZE_LG)
                                        .text(&format!("endpoints: {:?}", app.endpoints))
                                    }),
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
