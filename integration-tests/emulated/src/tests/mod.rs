//! Module for writing the actual integration tests of your solution.

use polkadot_sdk::xcm_emulator::assert_ok;

use super::prelude::*;
use crate::tests::xcm::opaque::latest::Junction::Parachain;
use parachain_runtime::RuntimeOrigin;
use polkadot_sdk::emulated_integration_tests_common::impls::Inspect;
use polkadot_sdk::{
    frame_support::traits::tokens::{fungible, fungibles},
    sp_runtime, staging_xcm as xcm,
};
use xcm::latest::Location;

const ALICE: u32 = 1;

// #[test]
// fn example_test() {
//     // This executes the inner code from the perspective of the custom parachain.
//     CustomPara::execute_with(|| {
//         assert_ok!(
//             <CustomPara as CustomParaPallet>::XcmUtils::register_native_asset_on_ah(
//                 RuntimeOrigin::root()
//             ),
//         );
//     });
//     // This executes the inner code from the perspective of Asset Hub.
//     AssetHubWestend::execute_with(|| {
//         let ah_registered_asset = Location::new(0, [Parachain(2000)]);
//     });
// }

#[test]
fn test_register_native_asset_creates_foreign_asset_on_ah() {
    // Execute registration from custom parachain
    CustomPara::execute_with(|| {
        assert_ok!(
            <CustomPara as CustomParaPallet>::XcmUtils::register_native_asset_on_ah(
                RuntimeOrigin::root()
            ),
        );
    });

    // Verify the foreign asset was created on Asset Hub
    AssetHubWestend::execute_with(|| {
        let parachain_location = Location::new(1, [Parachain(2000)]);

        // Check if the foreign asset exists
        assert!(
            <AssetHubWestend as AssetHubWestendPallet>::ForeignAssets::asset_exists(
                parachain_location
            )
        );
    });
}
