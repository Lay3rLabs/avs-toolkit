use cw_orch::prelude::MockBech32;

use super::common::BECH_PREFIX;

#[test]
fn happy_path_works() {
    let chain = MockBech32::new(BECH_PREFIX);
    super::common::happy_path(chain);
}

#[test]
fn require_quorum_works() {
    let chain = MockBech32::new(BECH_PREFIX);
    super::common::require_quorum(chain);
}
