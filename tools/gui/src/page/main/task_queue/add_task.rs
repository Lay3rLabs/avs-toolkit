use avs_toolkit_shared::{task_queue::TaskQueue, wasmatic::Trigger};
use dominator_helpers::futures::AsyncLoader;
use lavs_apis::{id::TaskId, time::Duration};

use crate::{
    page::main::wasmatic::{get_apps, AppEntry},
    prelude::*,
};

pub struct TaskQueueAddTaskUi {
    wasmatic_app_index: Mutable<Option<usize>>,
    wasmatic_apps: Mutable<Option<Vec<Arc<AppEntry>>>>,
    payload: Mutable<Option<serde_json::Value>>,
    description: Mutable<Option<String>>,
    timeout: Mutable<Option<Duration>>,
    add_task_loader: AsyncLoader,
    address_error: Mutable<Option<String>>,
    payload_error: Mutable<Option<String>>,
    exec_error: Mutable<Option<String>>,
    task_id: Mutable<Option<TaskId>>,
}

impl TaskQueueAddTaskUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            wasmatic_app_index: Mutable::new(None),
            wasmatic_apps: Mutable::new(None),
            add_task_loader: AsyncLoader::new(),
            address_error: Mutable::new(None),
            payload_error: Mutable::new(None),
            exec_error: Mutable::new(None),
            task_id: Mutable::new(None),
            payload: Mutable::new(None),
            description: Mutable::new(None),
            timeout: Mutable::new(None),
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
            .future(clone!(state => async move {
                match get_apps().await {
                    Ok(apps) => {
                        state.wasmatic_apps.set(Some(apps));
                    },
                    Err(err) => {
                        state.address_error.set(Some(err.to_string()));
                    }
                }
            }))
            .child_signal(state.wasmatic_apps.signal_cloned().map(clone!(state => move |apps| {
                Some(match apps {
                    None => html!("div", {
                        .class(FontSize::Header.class())
                        .text("Loading...")
                    }),
                    Some(apps) => state.render_apps(apps)
                })
            })))
        })
    }

    fn render_apps(self: &Arc<Self>, apps: Vec<Arc<AppEntry>>) -> Dom {
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
                .text("Add Task")
            }))
            .child(Label::new()
                .with_text("Wasmatic App")
                .with_direction(LabelDirection::Column)
                .render(
                    html!("div", {
                        .style("display", "inline-block")
                        .child(Dropdown::new()
                            .with_intial_selected(None)
                            .with_options(apps.iter().enumerate().map(|(index, entry)| {
                                (entry.app.name.clone(), index)
                            }))
                            .with_on_change(clone!(state => move |index| {
                                state.wasmatic_app_index.set(Some(*index));
                            }))
                            .render()
                        )
                    })
                )
            )
            .child(Label::new()
                .with_text("Payload")
                .with_direction(LabelDirection::Column)
                .render(TextArea::new()
                    .with_placeholder("e.g. {\"key\": \"value\"}")
                    .with_mixin(|dom| {
                        dom
                            .style("width", "30rem")
                            .style("height", "10rem")
                    })
                    .with_on_input(clone!(state => move |payload| {
                        state.payload_error.set_neq(None);

                        match payload {
                            None => {
                                state.payload.set_neq(None)
                            },
                            Some(payload) => {
                                match serde_json::from_str(&payload) {
                                    Err(err) => {
                                        state.payload_error.set(Some(err.to_string()));
                                    },
                                    Ok(payload) => {
                                        state.payload.set(Some(payload));
                                    }
                                }
                            }
                        }
                    }))
                    .render()
                )
            )
            .child(Label::new()
                .with_text("Description")
                .with_direction(LabelDirection::Column)
                .render(TextInput::new()
                    .with_placeholder("e.g. do the thing")
                    .with_on_input(clone!(state => move |description| {
                        state.description.set_neq(description);
                    }))
                    .render()
                )
            )
            .child_signal(state.address_error.signal_cloned().map(clone!(state => move |error| {
                match error {
                    None => None,
                    Some(error) => Some(html!("div", {
                        .class([FontSize::Header.class(), &*&*COLOR_TEXT_INTERACTIVE_ERROR])
                        .text(&error)
                    }))
                }
            })))
            .child_signal(state.payload_error.signal_cloned().map(clone!(state => move |error| {
                match error {
                    None => None,
                    Some(error) => Some(html!("div", {
                        .class([FontSize::Header.class(), &*&*COLOR_TEXT_INTERACTIVE_ERROR])
                        .text(&error)
                    }))
                }
            })))
            .child_signal(state.exec_error.signal_cloned().map(clone!(state => move |error| {
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
                .with_text("Add Task")
                .with_on_click(clone!(state => move || {
                    state.exec_error.set_neq(None);
                    match (
                        state.description.get_cloned(),
                        state.wasmatic_app_index.get(),
                    ) {
                        (Some(description), Some(wasmatic_app_index)) => {
                            let app = &apps[wasmatic_app_index];
                            let task_queue_addr = match &app.app.trigger {
                                Trigger::Queue { task_queue_addr, ..} => {
                                    match query_client().chain_config.parse_address(&task_queue_addr) {
                                        Ok(addr) => addr,
                                        Err(err) => {
                                            state.exec_error.set(Some(err.to_string()));
                                            return;
                                        }
                                    }
                                },
                                _ => {
                                    state.exec_error.set(Some("Error: You need to provide a task trigger".to_string()));
                                    return;
                                }
                            };
                            state.add_task_loader.load(clone!(state => async move {
                                let task_queue = TaskQueue::new(signing_client(), task_queue_addr).await;
                                let payload = state.payload.get_cloned().unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
                                let res = task_queue.add_task(payload, description, state.timeout.get_cloned(), None, None).await;

                                match res {
                                    Ok((task_id, tx_resp)) => {
                                        log::info!("Task added with id: {task_id}");
                                        log::info!("TX hash: {}", tx_resp.txhash);
                                        state.task_id.set(Some(task_id));
                                    },
                                    Err(err) => {
                                        state.exec_error.set(Some(err.to_string()));
                                    }
                                }
                            }))
                        }
                        _ => {}
                    }
                }))
                .render()
            )
            .child_signal(state.add_task_loader.is_loading().map(|loading| {
                if loading {
                    Some(html!("div", {
                        .class(FontSize::Header.class())
                        .text("Loading...")
                    }))
                } else {
                    None
                }
            }))
        })
    }

    fn disabled_signal(self: &Arc<Self>) -> impl Signal<Item = bool> {
        let state = self;
        map_ref! {
            let has_app_index = state.wasmatic_app_index.signal_ref(|x| x.is_some()),
            let has_description = state.description.signal_ref(|x| x.is_some()),
            let has_address_error = state.address_error.signal_ref(|x| x.is_some()),
            let has_payload_error = state.payload_error.signal_ref(|x| x.is_some()),
            let has_exec_error = state.exec_error.signal_ref(|x| x.is_some()),
            => {
                *has_address_error
                || *has_payload_error
                || *has_exec_error
                || !(*has_app_index && *has_description)
            }
        }
    }
}
