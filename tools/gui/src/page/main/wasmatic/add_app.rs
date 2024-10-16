mod envs;
mod permissions;
mod trigger;
mod wasm_source;

use crate::prelude::*;
use avs_toolkit_shared::{
    deploy::DeployContractAddrs,
    file::WasmFile,
    wasmatic::{self, Trigger},
};
use dominator_helpers::futures::AsyncLoader;
use envs::EnvsUi;
use layer_climb::proto::abci::TxResponse;
use permissions::PermissionsUi;
use trigger::{TriggerData, TriggerUi};
use wasm_bindgen_futures::JsFuture;
use wasm_source::WasmSourceUi;
use web_sys::{js_sys, File};

pub struct WasmaticAddAppUi {
    pub loader: AsyncLoader,
    pub error: Mutable<Option<String>>,
    pub success: Mutable<Option<String>>,
    pub form: WasmaticAddAppForm,
}

struct WasmaticAddAppForm {
    pub wasm_source: Arc<WasmSourceUi>,
    pub trigger: Arc<TriggerUi>,
    pub permissions: Arc<PermissionsUi>,
    pub envs: Arc<EnvsUi>,
    pub name: Mutable<Option<String>>,
    pub testable: Mutable<bool>,
}

impl WasmaticAddAppForm {
    pub fn new() -> Self {
        Self {
            wasm_source: WasmSourceUi::new(),
            trigger: TriggerUi::new(),
            permissions: PermissionsUi::new(),
            envs: EnvsUi::new(),
            name: Mutable::new(None),
            testable: Mutable::new(false),
        }
    }
}

impl WasmaticAddAppUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            loader: AsyncLoader::new(),
            error: Mutable::new(None),
            success: Mutable::new(None),
            form: WasmaticAddAppForm::new(),
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

        static BUTTON_AND_MISSING: LazyLock<String> = LazyLock::new(|| {
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
                .text("Add Wasmatic App")
            }))
            .child(Label::new()
                .with_text("Name")
                .with_direction(LabelDirection::Column)
                .render(TextInput::new()
                    .with_placeholder("Your app name..")
                    .with_mixin(|dom| {
                        dom
                            .style("width", "30rem")
                    })
                    .with_on_input(clone!(state => move |name| {
                        state.form.name.set_neq(name);
                    }))
                    .render()
                )
            )
            .child(state.form.wasm_source.render())
            .child(state.form.trigger.render())
            .child(state.form.permissions.render())
            .child(state.form.envs.render())
            .child(Checkbox::new()
                .with_label("Testable")
                .with_selected_signal(state.form.testable.signal())
                .with_on_click(clone!(state => move || {
                    state.form.testable.set_neq(!state.form.testable.get());
                }))
                .render()
            )
            .child(html!("div", {
                .class(&*BUTTON_AND_MISSING)
                .child(Button::new()
                    .with_text("Upload")
                    .with_disabled_signal(state.missing_signal().map(|missing| !missing.is_empty()))
                    .with_on_click(clone!(state => move || {
                        state.loader.load(clone!(state => async move {
                            state.error.set(None);
                            state.success.set(None);
                            match state.extract_form_data().await {
                                Ok(FormData { file, digest, trigger, name, permissions, envs, testable }) => {

                                    let trigger = match trigger {
                                        TriggerData::Cron { schedule } => Trigger::Cron { schedule },
                                        TriggerData::Queue { contract_args, hd_index, poll_interval } => {
                                            match DeployContractAddrs::run(signing_client(), contract_args).await {
                                                Ok(addrs) => {
                                                    Trigger::Queue {
                                                        task_queue_addr: addrs.task_queue.to_string(),
                                                        hd_index,
                                                        poll_interval,
                                                    }
                                                },
                                                Err(err) => {
                                                    state.error.set(Some(format!("Error deploying contract: {:?}", err)));
                                                    return;
                                                }
                                            }
                                        }
                                    };

                                    match wasmatic::deploy(
                                        http_client(),
                                        query_client(),
                                        CONFIG.chain_info().unwrap_ext().wasmatic.endpoints.clone(),
                                        name,
                                        digest,
                                        file,
                                        trigger,
                                        permissions,
                                        envs,
                                        testable,
                                        |endpoint| {
                                            log::info!("App added to: {endpoint}");
                                        },
                                    )
                                    .await {
                                        Ok(_) => {
                                            state.success.set(Some("App added to all endpoints".to_string()));
                                        }
                                        Err(err) => {
                                            state.error.set(Some(format!("Error adding app: {:?}", err)));
                                        }
                                    }
                                },
                                Err(err) => {
                                    state.error.set(Some(format!("Error reading file: {:?}", err)));
                                }
                            }
                        }));
                    }))
                    .render()
                )
                .child_signal(state.missing_signal().map(|missing| {
                    match missing.is_empty() {
                        true => None,
                        false => {
                            Some(html!("div", {
                                .class([FontSize::Body.class(), &*&*COLOR_TEXT_INTERACTIVE_ERROR])
                                .children(missing.iter().map(|missing| {
                                    html!("div", {
                                        .text(&format!("Missing: {}", missing))
                                    })
                                }))
                            }))
                        }
                    }
                }))

            }))
            .child_signal(state.loader.is_loading().map(|is_loading| {
                match is_loading {
                    true => Some(html!("div", {
                        .class(FontSize::Body.class())
                        .text("Uploading...")
                    })),
                    false => None
                }
            }))
            .child_signal(state.success.signal_cloned().map(|success| {
                match success {
                    Some(msg) => Some(html!("div", {
                        .child(html!("div", {
                            .class([FontSize::Body.class(), ColorText::Brand.color_class()])
                            .text(&msg)
                        }))
                    })),
                    None => None
                }
            }))
            .child_signal(state.error.signal_cloned().map(|error| {
                match error {
                    Some(error) => Some(html!("div", {
                        .class([FontSize::Body.class(), &*COLOR_TEXT_INTERACTIVE_ERROR])
                        .text(&error)
                    })),
                    None => None
                }
            }))
        })
    }

    async fn extract_form_data(self: &Arc<Self>) -> Result<FormData> {
        let state = self;

        let (file, digest) = state.form.wasm_source.extract().await?;
        let name = state.form.name.get_cloned().context("name is required")?;

        let trigger = state.form.trigger.extract().await?;
        let permissions = state
            .form
            .permissions
            .extract()
            .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));

        let testable = state.form.testable.get();

        let envs = state.form.envs.extract()?;

        Ok(FormData {
            file,
            digest,
            trigger,
            permissions,
            name,
            testable,
            envs,
        })
    }

    fn missing_signal(&self) -> impl Signal<Item = Vec<String>> {
        map_ref! {
            let wasm_source_valid = self.form.wasm_source.valid_signal(),
            let trigger_valid = self.form.trigger.valid_signal(),
            let has_name = self.form.name.signal_ref(|name| name.is_some()),
            let permissions_valid = self.form.permissions.valid_signal(),
            let envs_valid = self.form.envs.valid_signal(),
            => {
                let mut missing = Vec::new();
                if !wasm_source_valid {
                    missing.push("Wasm source".to_string());
                }
                if !trigger_valid {
                    missing.push("Trigger".to_string());
                }
                if !has_name {
                    missing.push("Name".to_string());
                }
                if !permissions_valid {
                    missing.push("Permissions".to_string());
                }
                if !envs_valid {
                    missing.push("Environment Variables".to_string());
                }

                missing
            }
        }
    }
}

struct FormData {
    file: WasmFile,
    digest: Option<String>,
    trigger: TriggerData,
    name: String,
    permissions: serde_json::Value,
    testable: bool,
    envs: Vec<(String, String)>,
}
