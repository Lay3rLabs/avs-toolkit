use crate::prelude::*;
use anyhow::Chain;
use layer_climb::prelude::*;
use std::sync::OnceLock;

pub static CLIENT: OnceLock<SigningClient> = OnceLock::new();
pub static ENVIRONMENT: OnceLock<TargetEnvironment> = OnceLock::new();
pub static FAUCET_CLIENT: OnceLock<SigningClient> = OnceLock::new();

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

pub async fn client_connect(key_kind: ClientKeyKind, target_env: TargetEnvironment) -> Result<()> {
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
            .context("local chain not configured")?
            .chain
            .clone(),
    }
    .into();

    let client = match key_kind {
        ClientKeyKind::DirectInput { mnemonic } => {
            let signer = KeySigner::new_mnemonic_str(&mnemonic, None)?;
            SigningClient::new(chain_config, signer).await?
        }

        ClientKeyKind::DirectEnv => {
            let env_str = match target_env {
                TargetEnvironment::Testnet => option_env!("TEST_MNEMONIC"),
                TargetEnvironment::Local => option_env!("LOCAL_MNEMONIC"),
            };

            let mnemonic = env_str.context("mnemonic not found in env")?;

            let signer = KeySigner::new_mnemonic_str(&mnemonic, None)?;

            SigningClient::new(chain_config, signer).await?
        }
        ClientKeyKind::Keplr => {
            let signer = KeplrSigner::new(&chain_config.chain_id).await?;
            SigningClient::new(chain_config, signer).await?
        }
    };

    log::info!("got client: {}", client.addr);

    CLIENT.set(client);
    ENVIRONMENT.set(target_env);
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
            .context("local chain not configured")?
            .chain
            .clone(),
    };

    KeplrSigner::add_chain(&chain_config).await
}
