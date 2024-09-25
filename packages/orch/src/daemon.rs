use cosmwasm_std::Addr;
use cw_orch::daemon::env::{
    LOCAL_MNEMONIC_ENV_NAME, MAIN_MNEMONIC_ENV_NAME, TEST_MNEMONIC_ENV_NAME,
};
use cw_orch::daemon::keys::private::PrivateKey;
use cw_orch::daemon::senders::CosmosSender;
use cw_orch::daemon::TxSender;
use cw_orch::env_vars::DaemonEnvVars;
use cw_orch::environment::{ChainInfoOwned, ChainKind};
use cw_orch::prelude::*;
use cw_orch_core::CwEnvError;
use secp256k1::{All, Secp256k1};

use crate::{Addressable, AltSigner};

impl Addressable for CosmosSender<All> {
    fn addr(&self) -> Addr {
        self.address()
    }
}

// much was taken and adopted from cw-orch-daemon code
// I don't want to merge it but rather have a simpler helper to do clone_with_options or something
fn new_pk(mnemonic: Option<String>, hd_index: u32, info: &ChainInfoOwned) -> PrivateKey {
    let secp = Secp256k1::new();
    let mnemonic = match mnemonic {
        Some(m) => m,
        None => get_mnemonic_env(&info.kind).unwrap(),
    };
    // FIXME: avoid unwrap
    PrivateKey::from_words(&secp, &mnemonic, 0, hd_index, info.network_info.coin_type).unwrap()
}

fn get_mnemonic_env(chain_kind: &ChainKind) -> Result<String, CwEnvError> {
    match chain_kind {
        ChainKind::Local => DaemonEnvVars::local_mnemonic(),
        ChainKind::Testnet => DaemonEnvVars::test_mnemonic(),
        ChainKind::Mainnet => DaemonEnvVars::main_mnemonic(),
        _ => None,
    }
    .ok_or(CwEnvError::EnvVarNotPresentNamed(
        get_mnemonic_env_name(chain_kind).to_string(),
    ))
}

fn get_mnemonic_env_name(chain_kind: &ChainKind) -> &str {
    match chain_kind {
        ChainKind::Local => LOCAL_MNEMONIC_ENV_NAME,
        ChainKind::Testnet => TEST_MNEMONIC_ENV_NAME,
        ChainKind::Mainnet => MAIN_MNEMONIC_ENV_NAME,
        _ => panic!("Can't set mnemonic for unspecified chainkind"),
    }
}

impl AltSigner for Daemon {
    fn alt_signer(&self, index: u32) -> Self::Sender {
        let mut new_sender = self.sender().clone();
        // TODO: reuse mnemonic if we aren't just pulling from env
        let pk = new_pk(None, index, self.chain_info());
        new_sender.private_key = pk;
        new_sender
    }
}

/// loads from env, sets up logging
pub fn daemon_setup() {
    // Used to load the `.env` file if any
    // Store TEST_MNEMONIC (and/or LOCAL_MNEMONIC) there
    dotenv::dotenv().ok();
    pretty_env_logger::init(); // Used to log contract and chain interactions
}

/// Creates a daemon connected to specified slay3r chain (local, devnet, or mainnet)
pub fn slay3r_connect(kind: ChainKind) -> Daemon {
    daemon_setup();
    let info = crate::networks::chain_info(kind);
    let chain = DaemonBuilder::new(info).build().unwrap();
    chain
}
