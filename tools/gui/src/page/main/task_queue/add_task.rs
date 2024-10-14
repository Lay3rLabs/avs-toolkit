use avs_toolkit_shared::task_queue::TaskQueue;
use dominator_helpers::futures::AsyncLoader;

use crate::{config::get_default_task_queue_addr, prelude::*};

pub struct TaskQueueAddTaskUi {
    task_queue_addr: Mutable<Option<Address>>,
    add_task_loader: AsyncLoader,
    error: Mutable<Option<String>>,
}

impl TaskQueueAddTaskUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            task_queue_addr: Mutable::new(get_default_task_queue_addr()),
            add_task_loader: AsyncLoader::new(),
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
                .with_text("Add Task")
                .with_on_click(clone!(state => move || {
                    if let Some(contract_addr) = state.task_queue_addr.get_cloned() {
                        state.add_task_loader.load(clone!(state => async move {
                            let task_queue = TaskQueue::new(signing_client(), contract_addr).await;
                        }))
                    }
                }))
                .render()
            )
            .child_signal(state.add_task_loader.is_loading().map(|loading| {
                if loading {
                    Some(html!("div", {
                        .class(&*TEXT_SIZE_LG)
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
            let has_task_queue_addr = state.task_queue_addr.signal_ref(|x| x.is_some()),
            let has_error = state.error.signal_ref(|x| x.is_some()),
            => {
                !has_task_queue_addr || *has_error
            }
        }
    }
}
