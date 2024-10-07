use crate::prelude::*;
use dominator_helpers::futures::AsyncLoader;
use futures::StreamExt;
use gloo_timers::future::IntervalStream;
use wasm_bindgen_futures::spawn_local;

pub struct WalletFaucetUi {
    pub balance: Mutable<u128>,
    pub client: SigningClient,
    pub loader: AsyncLoader,
}

impl WalletFaucetUi {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            balance: Mutable::new(0),
            loader: AsyncLoader::new(),
            client: CLIENT.get().unwrap_ext().clone(),
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
                state.update_balance().await;
                IntervalStream::new(3_000).for_each(clone!(state => move |_| clone!(state => async move {
                    state.update_balance().await;
                }))).await;
            }))
            .child(html!("div", {
                .class(&*TEXT_SIZE_XLG)
                .text_signal(state.balance.signal().map(clone!(state => move |balance| {
                    format!("Balance: {:.2}{}", balance, state.client.querier.chain_config.gas_denom)
                })))
            }))
            .child(html!("div", {
                .child(Button::new()
                    .with_text("Tap it!")
                    .with_on_click(clone!(state => move || {
                        state.loader.load(clone!(state => {
                            async move {
                                if let Err(err) = state.get_tokens().await {
                                    log::error!("Error getting tokens: {:?}", err);
                                }
                            }
                        }));
                    }))
                    .render()
                )
            }))
            .child_signal(state.loader.is_loading().map(|is_loading| {
                match is_loading {
                    true => Some(html!("div", {
                        .class(&*TEXT_SIZE_MD)
                        .text("Getting tokens...")
                    })),
                    false => None
                }
            }))
        })
    }

    async fn update_balance(&self) {
        self.balance.set_neq(
            self.client
                .querier
                .balance(self.client.addr.clone(), None)
                .await
                .unwrap_or_default()
                .unwrap_or_default(),
        );
    }

    async fn get_tokens(&self) -> Result<()> {
        let signer = KeySigner::new_mnemonic_str(&CONFIG.data.faucet.mnemonic, None)?;
        let faucet = SigningClient::new(self.client.querier.chain_config.clone(), signer).await?;

        faucet
            .transfer(1_000_000, &self.client.addr, None, None)
            .await?;

        self.update_balance().await;

        Ok(())
    }
}
