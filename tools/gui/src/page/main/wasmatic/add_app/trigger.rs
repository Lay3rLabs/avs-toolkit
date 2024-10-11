use avs_toolkit_shared::{file::WasmFile, wasmatic::Trigger};
use web_sys::File;

use crate::{config::get_default_task_queue_addr, prelude::*};

pub struct TriggerUi {
    selected: Mutable<TriggerChoice>,
    cron_job: Mutable<Option<String>>,
    task_address: Mutable<Option<Address>>,
    task_hd_index: Mutable<u32>,
    task_poll_interval_seconds: Mutable<u32>,
    error: Mutable<Option<String>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TriggerChoice {
    Cron,
    Task,
}

impl TriggerUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            selected: Mutable::new(TriggerChoice::Task),
            cron_job: Mutable::new(None),
            task_address: Mutable::new(get_default_task_queue_addr()),
            task_hd_index: Mutable::new(0),
            task_poll_interval_seconds: Mutable::new(3),
            error: Mutable::new(None),
        })
    }

    pub async fn extract(self: &Arc<Self>) -> Result<Trigger> {
        let state = self;

        match state.selected.get() {
            TriggerChoice::Task => {
                let address = state
                    .task_address
                    .get_cloned()
                    .ok_or_else(|| anyhow!("Task Queue contract address is required"))?;
                let hd_index = state.task_hd_index.get();
                let poll_interval = state.task_poll_interval_seconds.get();

                Ok(Trigger::Queue {
                    task_queue_addr: address.to_string(),
                    hd_index,
                    poll_interval,
                })
            }
            TriggerChoice::Cron => {
                let cron_job = state
                    .cron_job
                    .get_cloned()
                    .ok_or_else(|| anyhow!("Cron job name is required"))?;
                Ok(Trigger::Cron { schedule: cron_job })
            }
        }
    }

    pub fn valid_signal(self: &Arc<Self>) -> impl Signal<Item = bool> {
        let state = self;

        self.selected
            .signal()
            .map(clone!(state => move |choice| {
                match choice {
                    TriggerChoice::Task => {
                        state.task_address.signal_ref(|address| address.is_some()).boxed()
                    },
                    TriggerChoice::Cron => {
                        state.cron_job.signal_ref(|address| address.is_some()).boxed()
                    }
                }
            }))
            .flatten()
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
                .style("flex-direction", "row")
                .style("align-items", "flex-start")
                .style("gap", "1rem")
            }
        });

        let dropdown = Dropdown::new()
            .with_intial_selected(Some(state.selected.get_cloned()))
            .with_options([
                ("Task".to_string(), TriggerChoice::Task),
                ("Cron".to_string(), TriggerChoice::Cron),
            ])
            .with_on_change(clone!(state => move |choice| {
                state.error.set_neq(None);
                state.selected.set(*choice);
            }));

        html!("div", {
            .class(&*CONTAINER)
            .child(html!("div", {
                .class(&*ROW)
                .child(Label::new()
                    .with_text("Trigger")
                    .with_direction(LabelDirection::Column)
                    .render(dropdown.render())
                )
                .child_signal(state.selected.signal_cloned().map(clone!(state => move |choice| {
                    Some(match choice {
                        TriggerChoice::Task => {
                            state.render_task()
                        },
                        TriggerChoice::Cron => {
                            state.render_cron()
                        }
                    })
                })))
            }))
            .child_signal(state.error.signal_cloned().map(|error| {
                error.map(|error| {
                    html!("div", {
                        .class([&*TEXT_SIZE_MD, Color::Red.class()])
                        .text(&error)
                    })
                })
            }))
        })
    }

    fn render_task(self: &Arc<Self>) -> Dom {
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
            .child(Label::new()
                .with_text("Task Queue contract address")
                .with_direction(LabelDirection::Column)
                .render(TextInput::new()
                    .with_placeholder("e.g. slayaddr...")
                    .with_intial_value(state.task_address.get_cloned().map(|addr| addr.to_string()).unwrap_or_default())
                    .with_mixin(|dom| {
                        dom
                            .style("width", "30rem")
                    })
                    .with_on_input(clone!(state => move |address| {
                        state.error.set_neq(None);
                        match address {
                            None => {
                                state.task_address.set(None)
                            },
                            Some(address) => {
                                match query_client().chain_config.parse_address(&address) {
                                    Err(err) => {
                                        state.error.set(Some(err.to_string()));
                                    },
                                    Ok(address) => {
                                        state.task_address.set(Some(address));
                                    }
                                }
                            }
                        }
                    }))
                    .render()
                )
            )
            .child(Label::new()
                .with_text("HD index")
                .with_direction(LabelDirection::Column)
                .render(TextInput::new()
                    .with_kind(TextInputKind::Number)
                    .with_intial_value(state.task_hd_index.get())
                    .with_mixin(|dom| {
                        dom
                            .style("width", "5rem")
                            .style("padding", "0.5rem")
                    })
                    .with_on_input(clone!(state => move |hd_index| {
                        state.error.set_neq(None);
                        match hd_index {
                            None => {
                                state.task_hd_index.set(0)
                            },
                            Some(hd_index) => {
                                match hd_index.parse::<u32>() {
                                    Err(err) => {
                                        state.error.set(Some(err.to_string()));
                                    },
                                    Ok(hd_index) => {
                                        state.task_hd_index.set(hd_index);
                                    }
                                }
                            }
                        }
                    }))
                    .render()
                )
            )
            .child(Label::new()
                .with_text("Poll interval (seconds)")
                .with_direction(LabelDirection::Column)
                .render(TextInput::new()
                    .with_kind(TextInputKind::Number)
                    .with_intial_value(state.task_poll_interval_seconds.get())
                    .with_mixin(|dom| {
                        dom
                            .style("width", "5rem")
                            .style("padding", "0.5rem")
                    })
                    .with_on_input(clone!(state => move |seconds| {
                        state.error.set_neq(None);
                        match seconds {
                            None => {
                                state.task_poll_interval_seconds.set(0)
                            },
                            Some(seconds) => {
                                match seconds.parse::<u32>() {
                                    Err(err) => {
                                        state.error.set(Some(err.to_string()));
                                    },
                                    Ok(seconds) => {
                                        state.task_poll_interval_seconds.set(seconds);
                                    }
                                }
                            }
                        }
                    }))
                    .render()
                )
            )
        })
    }

    fn render_cron(self: &Arc<Self>) -> Dom {
        let state = self;

        Label::new()
            .with_text("Cron job")
            .with_direction(LabelDirection::Column)
            .render(
                TextInput::new()
                    .with_placeholder("some cron job name")
                    .with_on_input(clone!(state => move |cron_job| {
                        state.error.set_neq(None);
                        state.cron_job.set(cron_job);
                    }))
                    .render(),
            )
    }
}
