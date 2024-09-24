use cosmwasm_std::Addr;
use cw_orch::prelude::MockBech32;

use crate::{Addressable, AltSigner};

impl Addressable for Addr {
    fn addr(&self) -> Addr {
        self.clone()
    }
}

impl AltSigner for MockBech32 {
    fn alt_signer(&self, index: u32) -> Self::Sender {
        let name = format!("signer-{}", index);
        self.addr_make(name)
    }
}
