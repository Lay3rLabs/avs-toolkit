/* Disabled for now
use bitcoin::secp256k1::All;
use cosmwasm_std::Addr;
use cw_orch::daemon::sender::{Sender, SenderOptions};
use cw_orch::environment::{ChainKind, ChainState};
use cw_orch::prelude::*;

use crate::{Addressable, AltSigner};

impl Addressable for Sender<All> {
    fn addr(&self) -> Addr {
        self.address().unwrap()
    }
}

impl AltSigner for Daemon {
    fn alt_signer(&self, index: u32) -> Self::Sender {
        let options = SenderOptions::default().hd_index(index);
        // This is a bit mucking with internals... I had to look inside the DaemonBuilder...
        let state = self.state().chain_data.clone();
        let sender = Sender::<All>::new_with_options(state, self.channel(), options).unwrap();
        sender.into()
    }
}

/// loads from env, sets up logging
pub fn daemon_setup() {
    // Used to load the `.env` file if any
    // Store TEST_MNEMONIC (and/or LOCAL_MNEMONIC) there
    dotenv::dotenv().ok();
    pretty_env_logger::init(); // Used to log contract and chain interactions
}

/// Creates a daemon connected to specified layer chain (local, devnet, or mainnet)
pub fn layer_connect(kind: ChainKind) -> Daemon {
    daemon_setup();
    let info = crate::networks::chain_info(kind);
    let chain = DaemonBuilder::default().chain(info).build().unwrap();
    chain
}
*/
