use crate::{
    page::main::wasmatic::{get_apps, AppEntry},
    prelude::*,
};
use avs_toolkit_shared::{
    task_queue::{TaskQueue, TaskQueueView, TaskView},
    wasmatic::Trigger,
};
use dominator_helpers::futures::AsyncLoader;
use lavs_apis::tasks::TaskResponse;
use lavs_mock_operators::contract::query;

pub struct TaskQueueViewQueueUi {
    wasmatic_app_index: Mutable<Option<usize>>,
    wasmatic_apps: Mutable<Option<Vec<Arc<AppEntry>>>>,
    view_task_loader: AsyncLoader,
    error: Mutable<Option<String>>,
    result: Mutable<Option<(Address, TaskQueueView)>>,
}

impl TaskQueueViewQueueUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            wasmatic_app_index: Mutable::new(None),
            wasmatic_apps: Mutable::new(None),
            view_task_loader: AsyncLoader::new(),
            error: Mutable::new(None),
            result: Mutable::new(None),
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
                        state.error.set(Some(err.to_string()));
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

        static CONTENT: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("gap", "1rem")
            }
        });
        static RESULTS: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("margin-top", "1rem")
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("gap", ".5rem")
            }
        });

        static TASKS: LazyLock<String> = LazyLock::new(|| {
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
            .child_signal(state.error.signal_cloned().map(clone!(state => move |error| {
                match error {
                    None => None,
                    Some(error) => Some(html!("div", {
                        .class([FontSize::Header.class(), &*&*COLOR_TEXT_INTERACTIVE_ERROR])
                        .text(&error)
                    }))
                }
            })))
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
            .child(Button::new()
                .with_disabled_signal(state.disabled_signal())
                .with_text("View Queue")
                .with_on_click(clone!(state => move || {
                    if let Some(wasmatic_app_index) = state.wasmatic_app_index.get_cloned() {
                        let app = &apps[wasmatic_app_index];
                        let task_queue_addr = match &app.app.trigger {
                            Trigger::Queue { task_queue_addr, ..} => {
                                match query_client().chain_config.parse_address(&task_queue_addr) {
                                    Ok(addr) => addr,
                                    Err(err) => {
                                        state.error.set(Some(err.to_string()));
                                        return;
                                    }
                                }
                            },
                            _ => {
                                state.error.set(Some("Error: You need to provide a task trigger".to_string()));
                                return;
                            }
                        };
                        state.view_task_loader.load(clone!(state => async move {
                            state.error.set_neq(None);
                            state.result.set(None);

                            let task_queue = TaskQueue::new(signing_client(), task_queue_addr.clone()).await;

                            let res = task_queue
                                .querier
                                .task_queue_view(None, None)
                                .await;

                            match res {
                                Ok(result) => {
                                    state.error.set_neq(None);
                                    state.result.set(Some((task_queue_addr, result)));
                                },
                                Err(err) => {
                                    state.error.set(Some(err.to_string()));
                                }
                            }
                        }))
                    }
                }))
                .render()
            )
            .child_signal(state.view_task_loader.is_loading().map(|loading| {
                if loading {
                    Some(html!("div", {
                        .class(FontSize::Header.class())
                        .text("Loading...")
                    }))
                } else {
                    None
                }
            }))
            .child_signal(state.result.signal_cloned().map(|result| {
                match result {
                    None => None,
                    Some((task_queue_addr, result)) => Some(html!("div", {
                        .class([FontSize::Header.class(), &*CONTENT])
                        .children([
                            html!("div", {
                                .class(&*RESULTS)
                                .child(html!("div", {
                                    .text("Verifier Contract: ")
                                }))
                                .child(html!("div", {
                                    .class(FontWeight::Bold.class())
                                    .text(&result.verifier_addr.to_string())
                                }))
                            }),
                            html!("div", {
                                .class(&*RESULTS)
                                .child(html!("div", {
                                    .text("Operator Contract: ")
                                }))
                                .child(html!("div", {
                                    .class(FontWeight::Bold.class())
                                    .text(&result.operator_addr.to_string())
                                }))
                            }),
                            html!("div", {
                                .class(&*RESULTS)
                                .child(html!("div", {
                                    .text("Operators: ")
                                    .child(html!("ul", {
                                        .children(result.operators.iter().map(|operator| {
                                            html!("li", {
                                                .class(FontWeight::Bold.class())
                                                .text(&operator.address.to_string())
                                            })
                                        }))
                                    }))
                                }))
                            }),
                            html!("div", {
                                .class(&*RESULTS)
                                .child(html!("div", {
                                    .text("Tasks: ")
                                    .class(&*TASKS)
                                    .child(html!("ul", {
                                        .children(result.tasks.iter().map(|task| {
                                            html!("li", {
                                                .child(render_task(&task_queue_addr, task))
                                            })
                                        }))
                                    }))
                                }))
                            })
                        ])
                    }))
                }
            }))
        })
    }

    fn disabled_signal(self: &Arc<Self>) -> impl Signal<Item = bool> {
        let state = self;
        map_ref! {
            let has_wasmatic_index = state.wasmatic_app_index.signal_ref(|x| x.is_some()),
            let has_error = state.error.signal_ref(|x| x.is_some()),
            => {
                !has_wasmatic_index || *has_error
            }
        }
    }
}

fn render_task(task_queue_addr: &Address, task: &TaskView) -> Dom {
    static TASK: LazyLock<String> = LazyLock::new(|| {
        class! {
            .style("display", "flex")
            .style("flex-direction", "column")
            .style("gap", "1rem")
        }
    });

    let task_details: Mutable<Option<std::result::Result<TaskResponse, String>>> =
        Mutable::new(None);

    let task_id = task.id();

    html!("div", {
        .future(clone!(task_details, task_id, task_queue_addr => async move {
            let res:Result<TaskResponse> = query_client().contract_smart(&task_queue_addr, &lavs_task_queue::msg::QueryMsg::Custom(lavs_task_queue::msg::CustomQueryMsg::Task {
                id: task_id.clone(),
            })).await;

            match res {
                Ok(result) => {
                    task_details.set(Some(Ok(result)));
                },
                Err(err) => {
                    task_details.set(Some(Err(err.to_string())));
                }
            }
        }))
        .child_signal(task_details.signal_cloned().map(clone!(task => move |details| {
            let description = match &details {
                None => None,
                Some(Ok(details)) => Some(details.description.to_string()),
                Some(Err(err)) => Some(err.to_string())
            };

            let payload = match &details {
                None => None,
                Some(Ok(details)) => match serde_json::to_string_pretty(&details.payload) {
                    Ok(payload) => Some(payload),
                    Err(err) => Some(err.to_string())
                },
                Some(Err(err)) => Some(err.to_string())
            };
            Some(html!("div", {
                .class(&*TASK)
                .child(html!("div", {
                    .text(&format!("Id: {}", task.id()))
                }))
                .apply_if(description.is_some(), |dom| dom.child(html!("div", {
                    .text(&format!("Description: {}", description.unwrap_or_default()))
                })))
                .apply_if(payload.is_some(), |dom| dom.child(html!("div", {
                    .text(&format!("Payload: {}", payload.unwrap_or_default()))
                })))
                .child(html!("div", {
                    .text(&format!("Result: {}", task.data_json_string().unwrap_or_default()))
                }))
                .child(html!("div", {
                    .text(&format!("Status: {}", match &task {
                        TaskView::Open(task) => "Open",
                        TaskView::Completed(task) => "Completed",
                    }))
                }))
            }))
        })))
    })
}
