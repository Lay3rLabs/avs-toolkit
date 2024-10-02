use cw_orch::environment::{ChainKind, NetworkInfo};
use cw_orch::prelude::ChainInfo;

pub const LAYER_NETWORK: NetworkInfo = NetworkInfo {
    chain_name: "Layer",
    pub_address_prefix: "layer",
    coin_type: 118u32,
};

pub const LAYER_LOCAL: ChainInfo = ChainInfo {
    chain_id: "slay3r-local",
    gas_denom: "uslay",
    gas_price: 0.025,
    grpc_urls: &["http://localhost:9090"],
    lcd_url: Some("http://localhost:1317"),
    fcd_url: None,
    network_info: LAYER_NETWORK,
    kind: ChainKind::Local,
};

// pub const LAYER_DEV: ChainInfo = ChainInfo {
//     chain_id: "slay3r-dev",
//     gas_denom: "uslay",
//     gas_price: 0.025,
//     grpc_urls: &["https://grpc.slay3r.zone"],
//     lcd_url: Some("https://rpc.slay3r.zone"),
//     fcd_url: None,
//     network_info: LAYER_NETWORK,
//     kind: ChainKind::Testnet,
// };

pub const LAYER_DEV: ChainInfo = ChainInfo {
    chain_id: "slay3r-dev",
    gas_denom: "uslay",
    gas_price: 0.025,
    grpc_urls: &["https://grpc.dev-cav3.net"],
    lcd_url: Some("https://lcd.dev-cav3.net"),
    fcd_url: None,
    network_info: LAYER_NETWORK,
    kind: ChainKind::Testnet,
};

pub fn chain_info(kind: ChainKind) -> ChainInfo {
    match kind {
        ChainKind::Local => LAYER_LOCAL,
        ChainKind::Testnet => LAYER_DEV,
        ChainKind::Mainnet => panic!("Mainnet not supported"),
        ChainKind::Unspecified => panic!("Unspecified chain kind"),
    }
}
