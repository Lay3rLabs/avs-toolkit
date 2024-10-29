use std::str::FromStr;

use avs_toolkit_shared::{
    deploy::{
        CodeIds, DeployContractArgs, DeployContractArgsRequestor, DeployContractArgsVerifierMode,
    },
    file::WasmFile,
    wasmatic::Trigger,
};
use cosmwasm_std::Decimal;
use lavs_apis::time::Duration;
use lavs_mock_operators::state;
use web_sys::File;

use crate::{config::DefaultCodeIds, prelude::*};

pub struct TriggerUi {
    selected: Mutable<TriggerChoice>,
    cron_job: Mutable<Option<String>>,
    task_queue: TaskQueueArgs,
    error: Mutable<Option<String>>,
}

pub struct TaskQueueArgs {
    pub code_ids: Mutable<CodeIds>,
    pub task_hd_index: Mutable<u32>,
    pub task_poll_interval_seconds: Mutable<u32>,
    pub task_timeout: Mutable<Duration>,
    pub required_voting_percentage: Mutable<u32>,
    pub threshold_percentage: Mutable<Option<Decimal>>,
    pub allowed_spread: Mutable<Option<Decimal>>,
    pub slashable_spread: Mutable<Option<Decimal>>,
    pub operators: Mutable<Vec<String>>,
    pub requestor: Mutable<DeployContractArgsRequestor>,
    pub mode: Mutable<DeployContractArgsVerifierMode>,
}

impl TaskQueueArgs {
    pub fn new() -> Self {
        let code_ids = DefaultCodeIds::new().unwrap();
        Self {
            code_ids: Mutable::new(CodeIds {
                mock_operators: code_ids.mock_operators.unwrap_or_default(),
                task_queue: code_ids.task_queue.unwrap_or_default(),
                verifier_simple: code_ids.verifier_simple.unwrap_or_default(),
                verifier_oracle: code_ids.verifier_oracle.unwrap_or_default(),
            }),
            task_hd_index: Mutable::new(0),
            task_poll_interval_seconds: Mutable::new(3),
            task_timeout: Mutable::new(Duration::new_seconds(300)),
            required_voting_percentage: Mutable::new(66),
            threshold_percentage: Mutable::new(None),
            allowed_spread: Mutable::new(None),
            slashable_spread: Mutable::new(None),
            operators: Mutable::new(vec!["wasmatic".to_string()]),
            requestor: Mutable::new(DeployContractArgsRequestor::default()),
            mode: Mutable::new(DeployContractArgsVerifierMode::Simple),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TriggerChoice {
    Cron,
    Task,
}

#[derive(Debug, Clone)]
pub enum TriggerData {
    Cron {
        schedule: String,
    },
    Queue {
        contract_args: DeployContractArgs,
        hd_index: u32,
        poll_interval: u32,
    },
}

impl TriggerUi {
    pub fn new() -> Arc<Self> {
        let code_ids = DefaultCodeIds::new().unwrap();

        Arc::new(Self {
            selected: Mutable::new(TriggerChoice::Task),
            cron_job: Mutable::new(None),
            task_queue: TaskQueueArgs::new(),
            error: Mutable::new(None),
        })
    }

    pub async fn extract(self: &Arc<Self>) -> Result<TriggerData> {
        let state = self;

        match state.selected.get() {
            TriggerChoice::Task => {
                let code_ids = state.task_queue.code_ids.get_cloned();

                let contract_args = DeployContractArgs::parse(
                    http_client(),
                    signing_client(),
                    CONFIG.chain_info().unwrap_ext().wasmatic.endpoints.clone(),
                    code_ids,
                    None,
                    state.task_queue.task_timeout.get_cloned(),
                    state.task_queue.required_voting_percentage.get_cloned(),
                    state.task_queue.threshold_percentage.get_cloned(),
                    state.task_queue.allowed_spread.get_cloned(),
                    state.task_queue.slashable_spread.get_cloned(),
                    state.task_queue.operators.get_cloned(),
                    state.task_queue.requestor.get_cloned(),
                    state.task_queue.mode.get_cloned(),
                )
                .await?;

                let hd_index = state.task_queue.task_hd_index.get();
                let poll_interval = state.task_queue.task_poll_interval_seconds.get();

                Ok(TriggerData::Queue {
                    contract_args,
                    hd_index,
                    poll_interval,
                })
            }
            TriggerChoice::Cron => {
                let cron_job = state
                    .cron_job
                    .get_cloned()
                    .ok_or_else(|| anyhow!("Cron job name is required"))?;
                Ok(TriggerData::Cron { schedule: cron_job })
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
                        state.valid_task_queue_signal().boxed()
                    },
                    TriggerChoice::Cron => {
                        state.cron_job.signal_ref(|address| address.is_some()).boxed()
                    }
                }
            }))
            .flatten()
    }

    fn valid_task_queue_signal(self: &Arc<Self>) -> impl Signal<Item = bool> {
        let state = self;

        map_ref! {
            let code_ids = state.task_queue.code_ids.signal_cloned(),
            let operators = state.task_queue.operators.signal_cloned(),
            => {
                if code_ids.mock_operators == 0 || code_ids.task_queue == 0 || code_ids.verifier_simple == 0|| code_ids.verifier_oracle == 0 {
                    false
                } else if operators.is_empty() {
                    false
                } else {
                    true
                }
            }
        }
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
                        .class([FontSize::Body.class(), &*COLOR_TEXT_INTERACTIVE_ERROR])
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
        static ROW: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "row")
                .style("gap", "1rem")
            }
        });

        let code_ids = state.task_queue.code_ids.get_cloned();

        html!("div", {
            .class(&*CONTAINER)
            .child(html!("div", {
                .class(&*ROW)
                .child(state.render_from_str("Task queue code id", TextInputKind::Number, code_ids.task_queue, 0, clone!(state => move |code_id| {
                    state.task_queue.code_ids.lock_mut().task_queue = code_id;
                })))
                .child(state.render_from_str("Mock operators code id", TextInputKind::Number, code_ids.mock_operators, 0, clone!(state => move |code_id| {
                    state.task_queue.code_ids.lock_mut().mock_operators = code_id;
                })))
                .child(state.render_from_str("Verifier simple code id", TextInputKind::Number, code_ids.verifier_simple, 0, clone!(state => move |code_id| {
                    state.task_queue.code_ids.lock_mut().verifier_simple= code_id;
                })))
                .child(state.render_from_str("Verifier oracle code id", TextInputKind::Number, code_ids.verifier_oracle, 0, clone!(state => move |code_id| {
                    state.task_queue.code_ids.lock_mut().verifier_oracle = code_id;
                })))
            }))
            .child(html!("div", {
                .class(&*ROW)
                .child(state.render_from_str("HD index", TextInputKind::Number, state.task_queue.task_hd_index.get(), 0, clone!(state => move |value| {
                    state.task_queue.task_hd_index.set(value);
                })))
                .child(state.render_from_str("Poll interval", TextInputKind::Number, state.task_queue.task_poll_interval_seconds.get(), 0, clone!(state => move |value| {
                    state.task_queue.task_poll_interval_seconds.set(value);
                })))
            }))
            .child(html!("div", {
                .class(&*ROW)
                .child(state.render_from_str("Required voting percentage", TextInputKind::Number, state.task_queue.required_voting_percentage.get(), 0, clone!(state => move |value| {
                    state.task_queue.required_voting_percentage.set(value);
                })))
                .child(state.render_option_from_str("Threshhold percentage (oracle only)", TextInputKind::Number, state.task_queue.threshold_percentage.get(), clone!(state => move |value| {
                    state.task_queue.threshold_percentage.set(value);
                })))
                .child(state.render_option_from_str("Allowed spread (oracle only)", TextInputKind::Number, state.task_queue.allowed_spread.get(), clone!(state => move |value| {
                    state.task_queue.allowed_spread.set(value);
                })))
                .child(state.render_option_from_str("Slashable spread (oracle only)", TextInputKind::Number, state.task_queue.slashable_spread.get(), clone!(state => move |value| {
                    state.task_queue.slashable_spread.set(value);
                })))
            }))
            .child(state.render_from_str("Requestor (see CLI for examples)", TextInputKind::Text, state.task_queue.requestor.get_cloned(), DeployContractArgsRequestor::default(), clone!(state => move |value| {
                state.task_queue.requestor.set(value);
            })))
            .child(Label::new()
                .with_text("Verifier mode")
                .with_direction(LabelDirection::Column)
                .render(html!("div", {
                    .style("display", "inline-block")
                    .child(Dropdown::new()
                        .with_intial_selected(Some(state.task_queue.mode.get_cloned()))
                        .with_options([
                            ("Simple".to_string(), DeployContractArgsVerifierMode::Simple),
                            ("Oracle".to_string(), DeployContractArgsVerifierMode::Oracle),
                        ])
                        .with_on_change(clone!(state => move |mode| {
                            state.task_queue.mode.set(*mode);
                        }))
                        .render()
                    )
                }))
            )
            .child(Label::new()
                .with_text("Operators (see CLI for examples)")
                .with_direction(LabelDirection::Column)
                .render(TextArea::new()
                    .with_mixin(|dom| {
                        dom
                            .style("width", "30rem")
                            .style("height", "5rem")
                    })
                    .with_intial_value(state.task_queue.operators.get_cloned().join(", "))
                    .with_on_input(clone!(state => move |input| {
                        match input {
                            None => {
                                state.task_queue.operators.set(Vec::new());
                                state.error.set_neq(None);
                            },
                            Some(value) => {
                                let operators = value.split(',').map(|s| s.trim().to_string()).collect();
                                state.error.set_neq(None);
                                state.task_queue.operators.set(operators);
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

    fn render_from_str<T: FromStr + ToString + Clone + 'static>(
        self: &Arc<Self>,
        label: &str,
        kind: TextInputKind,
        initial: T,
        zero_value: T,
        on_input: impl Fn(T) + Clone + 'static,
    ) -> Dom {
        let state = self;
        Label::new()
            .with_text(label)
            .with_direction(LabelDirection::Column)
            .render(
                TextInput::new()
                    .with_kind(kind)
                    .with_intial_value(initial)
                    .with_on_input(clone!(state, on_input => move |code_id| {
                        state.error.set_neq(None);
                        match code_id {
                            None => {
                                on_input(zero_value.clone());
                            },
                            Some(value) => {
                                match value.parse::<T>() {
                                    Err(err) => {
                                        state.error.set(Some(format!("could not parse {value}")));
                                    },
                                    Ok(value) => {
                                        on_input(value);
                                    }
                                }
                            }
                        }
                    }))
                    .render(),
            )
    }

    fn render_option_from_str<T: FromStr + ToString + Clone + 'static>(
        self: &Arc<Self>,
        label: &str,
        kind: TextInputKind,
        initial: Option<T>,
        on_input: impl Fn(Option<T>) + Clone + 'static,
    ) -> Dom {
        let state = self;

        let mut input = TextInput::new().with_kind(kind).with_on_input(
            clone!(state, on_input => move |code_id| {
                state.error.set_neq(None);
                match code_id {
                    None => {
                        on_input(None);
                    },
                    Some(value) => {
                        match value.parse::<T>() {
                            Err(err) => {
                                state.error.set(Some(format!("could not parse {value}")));
                            },
                            Ok(value) => {
                                on_input(Some(value));
                            }
                        }
                    }
                }
            }),
        );

        if let Some(initial) = initial {
            input = input.with_intial_value(initial);
        }

        Label::new()
            .with_text(label)
            .with_direction(LabelDirection::Column)
            .render(input.render())
    }
}
