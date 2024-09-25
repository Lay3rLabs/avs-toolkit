use cw_orch::daemon::{DaemonBase, DaemonError, Wallet};
use cw_orch::environment::ChainKind;
use cw_orch::prelude::*;
use cw_orch::tokio::runtime::Handle;

// theoretically this should work to give async access to the daemon
// but... after passing it to contract interfaces, it doesn't seem to work
// i.e. there's no upload() method on the contract interface
// pub async fn slay3r_connect_async(kind: ChainKind) -> Result<DaemonAsyncBase<Wallet>, DaemonError> {
//     let info = crate::networks::chain_info(kind);
//     DaemonAsyncBuilder::new(info).build().await
// }

pub fn slay3r_connect(
    kind: ChainKind,
    handle: Option<&Handle>,
) -> Result<DaemonBase<Wallet>, DaemonError> {
    let info = crate::networks::chain_info(kind);
    let mut builder = DaemonBuilder::new(info);
    if let Some(handle) = handle {
        builder.handle(handle);
    }

    builder.build()
}
