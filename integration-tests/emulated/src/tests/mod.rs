//! Module for writing the actual integration tests of your solution.

use super::prelude::*;

#[test]
fn example_test() {
    // This executes the inner code from the perspective of the custom parachain.
    CustomPara::execute_with(|| {});
    // This executes the inner code from the perspective of Asset Hub.
    AssetHubWestend::execute_with(|| {});
}
