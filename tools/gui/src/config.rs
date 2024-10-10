use core::panic;
use cosmwasm_std::Addr;
use std::sync::OnceLock;

use crate::{
    client::{ClientKeyKind, TargetEnvironment},
    prelude::*,
    route::Route,
};

static TARGET_ENVIRONMENT: LazyLock<Mutex<Option<TargetEnvironment>>> =
    LazyLock::new(|| Mutex::new(None));

pub fn set_target_environment(target_env: TargetEnvironment) {
    TARGET_ENVIRONMENT.lock().unwrap().replace(target_env);
}

pub fn get_target_environment() -> Result<TargetEnvironment> {
    TARGET_ENVIRONMENT
        .lock()
        .unwrap()
        .as_ref()
        .cloned()
        .context("target environment not set")
}

cfg_if::cfg_if! {
    if #[cfg(feature = "debug")] {

        pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
            Config {
                root_path: "",
                media_root: "http://localhost:9000",
                data: serde_json::from_str(include_str!("../../config.json")).unwrap_ext(),
                debug: ConfigDebug::dev_mode(),
            }
        });
    } else {
        pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
            Config {
                root_path: "climb",
                media_root: "https://lay3rlabs.github.io/climb/media",
                data: serde_json::from_str(include_str!("../../config.json")).unwrap_ext(),
                debug: ConfigDebug::default(),
            }
        });
    }
}

#[derive(Debug)]
pub struct Config {
    // the part of the url that is not the domain
    // e.g. in http://example.com/foo/bar, this would be "foo" if we want
    // all parsing to start from /bar
    // it's helpful in shared hosting environments where the app is not at the root
    pub root_path: &'static str,
    pub media_root: &'static str,
    pub debug: ConfigDebug,
    pub data: ConfigData,
}

impl Config {
    pub fn app_image_url(&self, path: &str) -> String {
        format!("{}/{}", self.media_root, path)
    }

    pub fn chain_info(&self) -> Result<&ChainInfo> {
        match get_target_environment()? {
            TargetEnvironment::Local => self.data.local.as_ref(),
            TargetEnvironment::Testnet => self.data.testnet.as_ref(),
        }
        .context("chain info not found")
    }
}

#[derive(Debug)]
pub struct ConfigDebug {
    pub auto_connect: Option<ConfigDebugAutoConnect>,
    pub start_route: Mutex<Option<Route>>,
}

impl Default for ConfigDebug {
    fn default() -> Self {
        Self {
            auto_connect: None,
            start_route: Mutex::new(Some(Route::Wallet(WalletRoute::Faucet))),
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "autoconnect")] {
        impl ConfigDebug {
            pub fn dev_mode() -> Self {
                Self {
                    auto_connect: Some(ConfigDebugAutoConnect{
                        key_kind: ClientKeyKind::DirectEnv,
                        //key_kind: ClientKeyKind::Keplr,
                        target_env: TargetEnvironment::Local
                    }),
                    start_route: Mutex::new(Some(Route::Wasmatic(WasmaticRoute::List)))
                }
            }
        }
    } else {
        impl ConfigDebug {
            pub fn dev_mode() -> Self {
                Self {
                    auto_connect: None,
                    ..ConfigDebug::default()
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigDebugAutoConnect {
    pub key_kind: ClientKeyKind,
    pub target_env: TargetEnvironment,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigData {
    pub local: Option<ChainInfo>,
    pub testnet: Option<ChainInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChainInfo {
    pub chain: WebChainConfig,
    pub faucet: FaucetConfig,
    pub wasmatic: WasmaticConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WasmaticConfig {
    pub endpoints: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FaucetConfig {
    pub mnemonic: String,
}
