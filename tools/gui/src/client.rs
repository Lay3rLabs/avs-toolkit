use crate::{client, config::get_target_environment, prelude::*};
use async_broadcast::{broadcast, Receiver, Sender};
use futures::StreamExt;
use layer_climb::prelude::*;
use std::{borrow::BorrowMut, cell::RefCell, sync::OnceLock};
use wasm_bindgen_futures::spawn_local;

thread_local! {
    static SIGNING_CLIENT: RefCell<Option<Client>> = RefCell::new(None);
}

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| reqwest::Client::new());

static CLIENT_EVENTS: LazyLock<ClientEvents> = LazyLock::new(|| {
    let (sender, receiver) = broadcast(100);
    ClientEvents { sender, receiver }
});

pub fn http_client() -> reqwest::Client {
    HTTP_CLIENT.clone()
}

// this should always be called fresh, since the underlying public key can change with wallets
pub fn signing_client() -> SigningClient {
    SIGNING_CLIENT.with(|x| x.borrow().as_ref().unwrap_ext().signing().clone())
}

pub fn query_client() -> QueryClient {
    SIGNING_CLIENT.with(|x| x.borrow().as_ref().unwrap_ext().signing().querier.clone())
}

pub fn has_signing_client() -> bool {
    SIGNING_CLIENT.with(|x| x.borrow().is_some())
}

pub fn client_event_receiver() -> Receiver<ClientEvent> {
    CLIENT_EVENTS.receiver.clone()
}

#[derive(Debug, Clone)]
pub enum ClientEvent {
    AddressChanged,
}

enum Client {
    // Keplr needs to store the signer so it can keep the callbacks alive
    Keplr {
        client: SigningClient,
        signer: KeplrSigner,
    },
    Any {
        client: SigningClient,
    },
}

impl Client {
    pub fn signing(&self) -> &SigningClient {
        match self {
            Client::Keplr { client, .. } => client,
            Client::Any { client } => client,
        }
    }

    pub fn replace_signing(&mut self, client: SigningClient) {
        match self {
            Client::Keplr { client: old, .. } => *old = client,
            Client::Any { client: old } => *old = client,
        }
    }
}

struct ClientEvents {
    pub sender: Sender<ClientEvent>,
    pub receiver: Receiver<ClientEvent>,
}

#[derive(Debug, Clone)]
pub enum ClientKeyKind {
    DirectInput { mnemonic: String },
    Keplr,
    DirectEnv,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum TargetEnvironment {
    Testnet,
    Local,
}

pub async fn client_connect(key_kind: ClientKeyKind) -> Result<()> {
    let chain_config = CONFIG.chain_info()?.chain.clone().into();

    let client = match key_kind {
        ClientKeyKind::DirectInput { mnemonic } => {
            let signer = KeySigner::new_mnemonic_str(&mnemonic, None)?;
            Client::Any {
                client: SigningClient::new(chain_config, signer).await?,
            }
        }

        ClientKeyKind::DirectEnv => {
            let env_str = match get_target_environment()? {
                TargetEnvironment::Testnet => option_env!("TEST_MNEMONIC"),
                TargetEnvironment::Local => option_env!("LOCAL_MNEMONIC"),
            };

            let mnemonic = env_str.context("mnemonic not found in env")?;

            let signer = KeySigner::new_mnemonic_str(&mnemonic, None)?;

            Client::Any {
                client: SigningClient::new(chain_config, signer).await?,
            }
        }
        ClientKeyKind::Keplr => {
            let signer = KeplrSigner::new(&chain_config.chain_id, || {
                // account was changed - replace the signing client after refreshing its inner public key etc.
                spawn_local(async move {
                    let mut client = signing_client();

                    client.refresh_signer().await.unwrap_ext();

                    SIGNING_CLIENT
                        .with(|x| x.borrow_mut().as_mut().unwrap_ext().replace_signing(client));

                    // inform any listeners who want to know about it
                    CLIENT_EVENTS
                        .sender
                        .try_broadcast(ClientEvent::AddressChanged)
                        .unwrap_ext();
                });
            })
            .await?;

            Client::Keplr {
                client: SigningClient::new(chain_config, signer.inner.clone()).await?,
                signer,
            }
        }
    };

    log::info!("got client: {}", client.signing().addr);

    SIGNING_CLIENT.with(|x| *x.borrow_mut() = Some(client));
    Ok(())
}

pub async fn add_keplr_chain(target_env: TargetEnvironment) -> Result<()> {
    let chain_config = match target_env {
        TargetEnvironment::Testnet => CONFIG
            .data
            .testnet
            .as_ref()
            .context("testnet chain not configured")?
            .chain
            .clone(),
        TargetEnvironment::Local => CONFIG
            .data
            .local
            .as_ref()
            .context("testnet chain not configured")?
            .chain
            .clone(),
    };

    KeplrSigner::add_chain(&chain_config.into())
        .await
        .map_err(|e| e.into())
}
