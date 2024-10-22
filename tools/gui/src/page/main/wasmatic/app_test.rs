use avs_toolkit_shared::wasmatic::{self, TestResult};
use dominator_helpers::futures::AsyncLoader;

// this file isn't named `test_app` because `test_` prefix causes issues, in VSCode at least
use crate::{page::main::wasmatic::get_apps, prelude::*};

use super::AppEntry;

pub struct WasmaticTestAppUi {
    apps: Mutable<Option<Vec<Arc<AppEntry>>>>,
    selected_app: Mutable<Option<Arc<AppEntry>>>,
    error: Mutable<Option<String>>,
    payload: Mutex<Option<String>>,
    results: Mutable<Option<Vec<TestResult>>>,
    test_loader: AsyncLoader,
}

impl WasmaticTestAppUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            apps: Mutable::new(None),
            error: Mutable::new(None),
            selected_app: Mutable::new(None),
            payload: Mutex::new(None),
            results: Mutable::new(None),
            test_loader: AsyncLoader::new(),
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
            .child(html!("div", {
                .class(FontSize::Header.class())
                .style("margin-bottom", "1rem")
                .text("Current Wasmatic Apps")
            }))
            .future(clone!(state => async move {
                match get_apps().await {
                    Ok(apps) => {
                        state.apps.set(Some(apps));
                    },
                    Err(err) => {
                        state.error.set(Some(err.to_string()));
                    }
                }
            }))
            .child_signal(state.apps.signal_cloned().map(clone!(state => move |apps| {
                Some(match apps {
                    None => html!("div", { .text("Loading...") }),
                    Some(apps) => {
                        Label::new()
                        .with_text("App")
                        .render(Dropdown::new()
                            .with_options(apps.iter().enumerate().map(|(index, app)| {
                                (app.app.name.clone(), index)
                            }))
                            .with_on_change(clone!(state => move |index| {
                                state.selected_app.set(apps.get(*index).cloned())
                            }))
                            .render()
                        )
                    }
                })

            })))
            .child(
                Label::new()
                    .with_text("Payload")
                    .with_direction(LabelDirection::Column)
                    .render(TextArea::new()
                        .with_mixin(|dom| {
                            dom
                                .style("width", "30rem")
                                .style("height", "10rem")
                        })
                        .with_placeholder("Enter input here")
                        .with_on_input(clone!(state => move |text| {
                            *state.payload.lock().unwrap_ext() = text;
                            state.evaluate();
                        }))
                        .render()
                    )
            )
            .child_signal(state.error.signal_cloned().map(clone!(state => move |error| {
                match error {
                    None => None,
                    Some(error) => Some(html!("div", {
                        .class([FontSize::Header.class(), &*&*COLOR_TEXT_INTERACTIVE_ERROR])
                        .text(&error)
                    }))
                }
            })))
            .child(Button::new()
                .with_disabled_signal(state.disabled_signal())
                .with_text("Test")
                .with_on_click(clone!(state => move || {
                    if let Some(app) = state.selected_app.get_cloned() {
                        state.test_loader.load(clone!(state => async move {
                            state.error.set_neq(None);
                            state.results.set_neq(None);

                            let res = wasmatic::test(
                                http_client(),
                                app.endpoints.clone(),
                                app.app.name.clone(),
                                state.payload.lock().unwrap_ext().clone(),
                                |test_result| {
                                    log::info!("test_result: {:?}", test_result);
                                }
                            ).await;

                            match res {
                                Ok(results) => {
                                    state.error.set_neq(None);
                                    state.results.set(Some(results));
                                },
                                Err(err) => {
                                    state.error.set(Some(err.to_string()));
                                    state.results.set_neq(None);
                                }
                            }
                        }));
                    }
                }))
                .render()
            )
            .child_signal(state.test_loader.is_loading().map(|loading| {
                if loading {
                    Some(html!("div", {
                        .class(FontSize::Header.class())
                        .text("Testing...")
                    }))
                } else {
                    None
                }
            }))
            .child_signal(state.results.signal_cloned().map(|results| {
                match results {
                    None => None,
                    Some(results) => {
                        Some(html!("div", {
                            .class(&*CONTAINER)
                            .children(results.iter().map(|TestResult { endpoint, response_text }| {
                                html!("div", {
                                    .class(&*CONTAINER)
                                    .children([
                                        html!("div", {
                                            .text(&format!("Endpoint: {}", endpoint))
                                        }),
                                        html!("div", {
                                            .text("Response:")
                                        }),
                                        html!("div", {
                                            .text(&format!("{}", response_text))
                                        }),
                                    ])
                                })
                            }))
                        }))
                    }
                }
            }))
        })
    }

    fn disabled_signal(self: &Arc<Self>) -> impl Signal<Item = bool> {
        let state = self;
        map_ref! {
            let has_error = state.error.signal_ref(|x| x.is_some()),
            let has_selected_app = state.selected_app.signal_ref(|x| x.is_some())
            => {
                *has_error || !*has_selected_app
            }
        }
    }

    fn evaluate(self: &Arc<Self>) {
        let state = self;

        let err = match state.payload.lock().as_ref().unwrap_ext().as_ref() {
            None => None,
            Some(text) => {
                if text.is_empty() {
                    None
                } else {
                    match serde_json::from_str::<serde_json::Value>(text) {
                        Ok(_) => None,
                        Err(err) => Some(err.to_string()),
                    }
                }
            }
        };

        state.error.set_neq(err);
    }
}
