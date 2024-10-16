use crate::{
    client::{add_keplr_chain, client_connect, ClientKeyKind, TargetEnvironment},
    config::set_target_environment,
    prelude::*,
};

pub struct LandingUi {
    wallet_connected: Mutable<bool>,
    client_key_kind: Arc<Mutex<Option<ClientKeyKind>>>,
    target_environment: Mutable<Option<TargetEnvironment>>,
    error: Mutable<Option<String>>,
    phase: Mutable<Phase>,
}

impl LandingUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            wallet_connected: Mutable::new(false),
            client_key_kind: Arc::new(Mutex::new(
                CONFIG
                    .debug
                    .auto_connect
                    .as_ref()
                    .map(|x| x.key_kind.clone()),
            )),
            target_environment: Mutable::new(
                CONFIG
                    .debug
                    .auto_connect
                    .as_ref()
                    .map(|x| x.target_env.clone()),
            ),
            error: Mutable::new(None),
            phase: Mutable::new(Phase::Init),
        })
    }

    pub fn render(self: &Arc<Self>) -> Dom {
        let state = self;

        static CONTAINER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("width", "100%")
                .style("height", "100%")
                .style_signal("background-color", ColorBackground::Base.signal())
                .style_signal("color", ColorText::Body.signal())
            }
        });
        static CONTENT: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("justify-content", "center")
                .style("align-items", "center")
                .style("margin-top", "5rem")
                .style("gap", "1rem")
            }
        });

        html!("div", {
            .class(&*CONTAINER)
            .child(html!("div", {
                .future(state.wallet_connected.signal().for_each(|connected| {
                    async move {
                        // for debugging, we want to jump to an initial page, but:
                        // 1. only consider it after connection status has settled
                        // 2. only one time (not if we intentionally come back to landing)
                        if connected {
                            let start_route = CONFIG.debug.start_route.lock().unwrap_ext().take();
                            log::info!("Starting at route: {:?}", start_route);
                            if let Some(start_route) = start_route {
                                start_route.go_to_url();
                            }
                        }
                    }
                }))
                .child(html!("div", {
                    //.class(&*CONTENT)
                    .child(html!("div", {
                        .style("padding-top", "5rem")
                        .class([FontSize::Hero.class(), FontWeight::Bold.class(), &*TEXT_ALIGN_CENTER])
                        .text("Lay3r Demo")
                    }))
                    .child(html!("div", {
                        .child_signal(state.wallet_connected.signal().map(clone!(state => move |connected| {
                            if !connected {
                                Some(state.render_connect())
                            } else {
                                // this will only be shown temporarily
                                None
                            }
                        })))
                    }))
                }))
            }))
        })
    }

    fn render_connect(self: &Arc<Self>) -> Dom {
        let state = self;

        html!("div", {
            .future(state.phase.signal_cloned().for_each(clone!(state => move |phase_value| {
                clone!(state => async move {
                    match phase_value {
                        Phase::Init => {
                            if state.client_key_kind.lock().unwrap_ext().is_some() && state.target_environment.lock_ref().is_some() {
                                state.phase.set_neq(Phase::Connecting);
                            }
                        },
                        Phase::Connecting => {
                            // these unwrapped values are guaranteed, via UI blocking, to exist here
                            set_target_environment(state.target_environment.get_cloned().unwrap_ext());
                            let res = client_connect(
                                state.client_key_kind.lock().unwrap_ext().clone().unwrap_ext(),
                            ).await;

                            match res {
                                Ok(_) => {
                                    state.wallet_connected.set(true);
                                },
                                Err(e) => {
                                    log::error!("Error connecting: {:?}", e);

                                    match state.client_key_kind.lock().unwrap_ext().as_ref().unwrap_ext() {
                                        ClientKeyKind::DirectEnv => {
                                            state.error.set(Some("Unable to connect".to_string()));
                                        },
                                        ClientKeyKind::DirectInput { .. } => {
                                            state.error.set(Some("Unable to connect".to_string()));
                                        },
                                        ClientKeyKind::Keplr => {
                                            if let Some(keplr_err) = e.downcast_ref::<KeplrError>() {
                                                match keplr_err {
                                                    KeplrError::MissingChain => {
                                                        state.phase.set(Phase::MissingKeplrChain);
                                                    },
                                                    KeplrError::NoExist => {
                                                        state.phase.set(Phase::KeplrError("Couldn't find Keplr, have you installed the extension?".to_string()));
                                                    },
                                                    KeplrError::FailedEnable => {
                                                        // not really right... maybe keplr updated their error strings?
                                                        state.phase.set(Phase::MissingKeplrChain);
                                                        //state.phase.set(Phase::KeplrError("Failed to enable Keplr, if you cancelled - just try again".to_string()));
                                                    },
                                                    _ => {
                                                        state.phase.set(Phase::KeplrError(e.to_string()));
                                                    }
                                                }
                                            } else {
                                                state.phase.set(Phase::KeplrError(e.to_string()));
                                            }
                                        }
                                    }
                                }
                            }
                        },

                        Phase::KeplrError(_) | Phase::MissingKeplrChain => {
                            // do nothing, waiting for user to hit button to add keplr
                        },

                        Phase::InstallingKeplr => {

                            match add_keplr_chain(state.target_environment.get_cloned().unwrap_ext()).await {
                                Ok(_) => {
                                    state.phase.set(Phase::Connecting);
                                },
                                Err(e) => {
                                    log::error!("Error adding Keplr chain: {:?}", e);
                                    state.error.set(Some("Unable to add Keplr chain".to_string()));
                                }
                            }
                        }
                    }
                })
            })))
            .style("display", "flex")
            .style("justify-content", "center")
            .style("align-items", "center")
            .style("gap", "1rem")
            .child_signal(state.phase.signal_cloned().map(clone!(state => move |phase_value| {
                Some(match phase_value {
                    Phase::Init => {
                        state.render_wallet_select(None)
                    },
                    Phase::Connecting => {
                        html!("div", {
                            .class(FontSize::Header.class())
                            .text("Connecting...")
                        })
                    },
                    Phase::KeplrError(e) => {
                        state.render_wallet_select(Some(e))
                    }
                    Phase::MissingKeplrChain => {
                        state.render_missing_keplr_chain()
                    },
                    Phase::InstallingKeplr => {
                        html!("div", {
                            .class(FontSize::Header.class())
                            .text("Installing Keplr...")
                        })
                    },
                })
            })))
        })
    }

    fn render_wallet_select(self: &Arc<Self>, error: Option<String>) -> Dom {
        let state = self;

        static CONTAINER: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("align-items", "center")
                .style("gap", "1rem")
            }
        });
        static DROPDOWNS: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("gap", "1rem")
            }
        });

        #[derive(PartialEq, Clone, Copy, Debug)]
        enum SignerKind {
            Mnemonic,
            Keplr,
        }

        let signer_kind: Mutable<Option<SignerKind>> = Mutable::new(None);

        let disabled_connect_signal = map_ref! {
            let signer_kind = signer_kind.signal(),
            let target_env = state.target_environment.signal() => {
                signer_kind.is_none() || target_env.is_none()
            }
        };

        html!("div", {
            .class(&*CONTAINER)
            .child(html!("div", {
                .class(&*DROPDOWNS)
                .children([
                    Label::new()
                        .with_text("Signer")
                        .render(Dropdown::new()
                            .with_intial_selected(signer_kind.get_cloned())
                            .with_options([
                                ("Mnemonic".to_string(), SignerKind::Mnemonic),
                                ("Keplr".to_string(), SignerKind::Keplr),
                            ])
                            .with_on_change(clone!(state, signer_kind => move |signer| {
                                match signer {
                                    SignerKind::Mnemonic => {
                                        *state.client_key_kind.lock().unwrap_ext() = Some(ClientKeyKind::DirectInput {
                                            mnemonic: "".to_string()
                                        });
                                    },
                                    SignerKind::Keplr => {
                                        *state.client_key_kind.lock().unwrap_ext() = Some(ClientKeyKind::Keplr);
                                        signer_kind.set(Some(SignerKind::Keplr));
                                    },
                                }
                                signer_kind.set(Some(*signer));

                            }))
                            .render()
                        ),

                    Label::new()
                        .with_text("Target Environment")
                        .render(Dropdown::new()
                        .with_intial_selected(state.target_environment.get_cloned())
                        .with_options([
                            ("Local".to_string(), TargetEnvironment::Local),
                            ("Testnet".to_string(), TargetEnvironment::Testnet),
                        ])
                        .with_on_change(clone!(state => move |target_env| {
                            state.target_environment.set(Some(*target_env));
                        }))
                        .render()
                    )
                ])
            }))
            .child_signal(signer_kind.signal().map(clone!(state => move |signer_kind| {
                match signer_kind {
                    Some(SignerKind::Mnemonic) => {
                        Some(TextArea::new()
                            .with_placeholder("Mnemonic")
                            .with_on_input(clone!(state => move |mnemonic| {
                                *state.client_key_kind.lock().unwrap_ext() = Some(ClientKeyKind::DirectInput { mnemonic: mnemonic.unwrap_or_default() });
                            }))
                            .with_mixin(|dom| {
                                dom
                                    .class(FontSize::Body.class())
                                    .style("max-width", "90%")
                                    .style("width", "40rem")
                                    .style("height", "10rem")
                            })
                            .render()
                        )
                    },
                    Some(SignerKind::Keplr) | None => None
                }
            })))
            .child(Button::new()
                .with_text("Connect")
                .with_disabled_signal(disabled_connect_signal)
                .with_on_click(clone!(state => move || {
                    state.phase.set(Phase::Connecting);
                }))
                .render()
            )
            .apply_if(error.is_some(), |dom| {
                dom.child(html!("div", {
                    .style("margin-top", "1rem")
                    .class([FontSize::Body.class(), &*COLOR_TEXT_INTERACTIVE_ERROR])
                    .text(error.as_ref().unwrap_ext())
                }))
            })
        })
    }

    fn render_missing_keplr_chain(self: &Arc<Self>) -> Dom {
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
                .text("Keplr doesn't know about this chain yet...")
            }))
            .child(Button::new()
                .with_text("Add Keplr")
                .with_on_click(clone!(state => move || {
                    state.phase.set(Phase::InstallingKeplr);
                }))
                .render()
            )
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Phase {
    Init,
    Connecting,
    MissingKeplrChain,
    InstallingKeplr,
    KeplrError(String),
}
