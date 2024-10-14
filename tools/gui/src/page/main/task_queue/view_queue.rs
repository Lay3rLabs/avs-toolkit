use crate::{config::get_default_task_queue_addr, prelude::*};
use avs_toolkit_shared::task_queue::{TaskQueue, TaskQueueView, TaskView};
use dominator_helpers::futures::AsyncLoader;

pub struct TaskQueueViewQueueUi {
    task_queue_addr: Mutable<Option<Address>>,
    view_task_loader: AsyncLoader,
    error: Mutable<Option<String>>,
    result: Mutable<Option<TaskQueueView>>,
}

impl TaskQueueViewQueueUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            task_queue_addr: Mutable::new(get_default_task_queue_addr()),
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

        static TASK: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("gap", "1rem")
            }
        });

        html!("div", {
            .class(&*CONTAINER)
            .child(html!("div", {
                .class(&*TEXT_SIZE_LG)
                .text("Add Task")
            }))
            .child(Label::new()
                .with_text("Task Queue contract address")
                .with_direction(LabelDirection::Column)
                .render(TextInput::new()
                    .with_placeholder("e.g. slayaddr...")
                    .with_intial_value(state.task_queue_addr.lock_ref().as_ref().map(|addr| addr.to_string()).unwrap_or_default())
                    .with_mixin(|dom| {
                        dom
                            .style("width", "30rem")
                    })
                    .with_on_input(clone!(state => move |address| {
                        state.error.set_neq(None);
                        match address {
                            None => {
                                state.task_queue_addr.set_neq(None);
                            },
                            Some(address) => {
                                match query_client().chain_config.parse_address(&address) {
                                    Err(err) => {
                                        state.error.set(Some(err.to_string()));
                                    },
                                    Ok(address) => {
                                        state.task_queue_addr.set_neq(Some(address));
                                    }
                                }
                            }
                        }
                    }))
                    .render()
                )
            )
            .child_signal(state.error.signal_cloned().map(clone!(state => move |error| {
                match error {
                    None => None,
                    Some(error) => Some(html!("div", {
                        .class([&*TEXT_SIZE_LG, &*Color::Red.class()])
                        .text(&error)
                    }))
                }
            })))
            .child(Button::new()
                .with_disabled_signal(state.disabled_signal())
                .with_text("View Queue")
                .with_on_click(clone!(state => move || {
                    if let Some(contract_addr) = state.task_queue_addr.get_cloned() {
                        state.view_task_loader.load(clone!(state => async move {
                            state.error.set_neq(None);
                            state.result.set(None);

                            let task_queue = TaskQueue::new(signing_client(), contract_addr).await;

                            let res = task_queue
                                .querier
                                .task_queue_view(None, None)
                                .await;

                            match res {
                                Ok(result) => {
                                    state.error.set_neq(None);
                                    state.result.set(Some(result));
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
                        .class(&*TEXT_SIZE_LG)
                        .text("Loading...")
                    }))
                } else {
                    None
                }
            }))
            .child_signal(state.result.signal_cloned().map(|result| {
                match result {
                    None => None,
                    Some(result) => Some(html!("div", {
                        .class([&*TEXT_SIZE_LG, &*CONTENT])
                        .children([
                            html!("div", {
                                .class(&*RESULTS)
                                .child(html!("div", {
                                    .text("Verifier Contract: ")
                                }))
                                .child(html!("div", {
                                    .class(&*TEXT_WEIGHT_BOLD)
                                    .text(&result.verifier_addr.to_string())
                                }))
                            }),
                            html!("div", {
                                .class(&*RESULTS)
                                .child(html!("div", {
                                    .text("Operator Contract: ")
                                }))
                                .child(html!("div", {
                                    .class(&*TEXT_WEIGHT_BOLD)
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
                                                .class(&*TEXT_WEIGHT_BOLD)
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
                                                .class(&*TASK)
                                                .child(html!("div", {
                                                    .text(&format!("Task Status: {}", match task {
                                                        TaskView::Open(task) => "Open",
                                                        TaskView::Completed(task) => "Completed",
                                                    }))
                                                }))
                                                .child(html!("div", {
                                                    .text(&format!("Task Data: {}", task.data_json_string().unwrap_or_default()))
                                                }))
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
            let has_task_queue_addr = state.task_queue_addr.signal_ref(|x| x.is_some()),
            let has_error = state.error.signal_ref(|x| x.is_some()),
            => {
                !has_task_queue_addr || *has_error
            }
        }
    }
}
