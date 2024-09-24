use cosmwasm_std::Addr;
use cw_orch::environment::CwEnv;
use std::sync::Arc;

pub trait Addressable {
    fn addr(&self) -> Addr;
}

pub trait AltSigner: CwEnv {
    fn alt_signer(&self, index: u32) -> Self::Sender;
}

impl<T: Addressable> Addressable for Arc<T> {
    fn addr(&self) -> Addr {
        self.as_ref().addr()
    }
}
