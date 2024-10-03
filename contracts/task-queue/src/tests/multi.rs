// use cw_orch::environment::IndexResponse;
use cw_orch::prelude::*;

use crate::interface::Contract;
use crate::msg::{InstantiateMsg, Requestor, TimeoutInfo};

// TODO: shared variable
const BECH_PREFIX: &str = "layer";

// Note: there is an assumption of 5 second blocks in the test framework
// let's make this clear in the tests
const BLOCK_TIME: u64 = 5;

#[test]
fn happy_path_works() {
    let chain = MockBech32::new(BECH_PREFIX);
    super::common::happy_path(chain);
}

#[test]
fn crate_works() {
    let chain = MockBech32::new(BECH_PREFIX);
    super::common::create(chain);
}

#[test]
fn list_open_queries_with_expiration() {
    let chain = MockBech32::new(BECH_PREFIX);
    super::common::list_open_queries_with_expiration(chain, BLOCK_TIME, 0);
}

#[test]
fn completion_works() {
    let chain = MockBech32::new(BECH_PREFIX);
    super::common::completion_works(chain);
}

#[test]
fn task_status() {
    let chain = MockBech32::new(BECH_PREFIX);
    super::common::task_status_works(chain);
}

#[test]
fn task_pagination() {
    let chain = MockBech32::new(BECH_PREFIX);
    super::common::task_pagination_works(chain);
}

/// This is the simplest, most explicit test to bootstrap, before importing from common
#[test]
fn sanity_check() {
    let mock = MockBech32::new(BECH_PREFIX);
    let tasker = Contract::new(mock.clone());
    let code_id = tasker.upload().unwrap().uploaded_code_id().unwrap();
    assert_eq!(code_id, tasker.code_id().unwrap());

    let verifier = mock.addr_make("verifier");

    let msg = InstantiateMsg {
        requestor: Requestor::Fixed(mock.sender_addr().into()),
        timeout: TimeoutInfo {
            default: 3600,
            minimum: None,
            maximum: None,
        },
        verifier: verifier.to_string(),
    };
    let init_res = tasker.instantiate(&msg, None, &[]).unwrap();
    let contract_addr = init_res.instantiated_contract_address().unwrap();
    assert_eq!(contract_addr, tasker.address().unwrap());
}
