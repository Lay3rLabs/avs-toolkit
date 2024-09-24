use cw_orch::environment::{ChainKind, NetworkInfo};
use cw_orch::prelude::ChainInfo;

pub const SLAY3R_NETWORK: NetworkInfo = NetworkInfo {
    chain_name: "slay3r",
    pub_address_prefix: "slay3r",
    coin_type: 118u32,
};

pub const SLAY3R_LOCAL: ChainInfo = ChainInfo {
    chain_id: "slay3r-local",
    gas_denom: "uslay",
    gas_price: 0.025,
    grpc_urls: &["http://localhost:9090"],
    lcd_url: Some("http://localhost:1317"),
    fcd_url: None,
    network_info: SLAY3R_NETWORK,
    kind: ChainKind::Local,
};

// pub const SLAY3R_DEV: ChainInfo = ChainInfo {
//     chain_id: "slay3r-dev",
//     gas_denom: "uslay",
//     gas_price: 0.025,
//     grpc_urls: &["https://grpc.slay3r.zone"],
//     lcd_url: Some("https://rpc.slay3r.zone"),
//     fcd_url: None,
//     network_info: SLAY3R_NETWORK,
//     kind: ChainKind::Testnet,
// };

pub const SLAY3R_DEV: ChainInfo = ChainInfo {
    chain_id: "slay3r-dev",
    gas_denom: "uslay",
    gas_price: 0.025,
    grpc_urls: &["https://grpc.dev-cav3.net"],
    lcd_url: Some("https://lcd.dev-cav3.net"),
    fcd_url: None,
    network_info: SLAY3R_NETWORK,
    kind: ChainKind::Testnet,
};

pub fn chain_info(kind: ChainKind) -> ChainInfo {
    match kind {
        ChainKind::Local => SLAY3R_LOCAL,
        ChainKind::Testnet => SLAY3R_DEV,
        ChainKind::Mainnet => panic!("Mainnet not supported"),
    }
}
