use cw_orch::prelude::MockBech32;

use super::common::BECH_PREFIX;

#[test]
fn happy_path_works() {
    let chain = MockBech32::new(BECH_PREFIX);
    super::common::happy_path(chain);
}

#[test]
fn threshold_not_met() {
    let chain = MockBech32::new(BECH_PREFIX);
    super::common::threshold_not_met(chain);
}
