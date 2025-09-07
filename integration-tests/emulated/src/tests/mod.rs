//! Module for writing the actual integration tests of your solution.

use polkadot_sdk::xcm_emulator::assert_ok;

use super::prelude::*;
use crate::tests::xcm::opaque::latest::Junction::Parachain;
use parachain_runtime::RuntimeOrigin;
use polkadot_sdk::cumulus_primitives_core::ParaId;
use polkadot_sdk::emulated_integration_tests_common::impls::Inspect;
use polkadot_sdk::polkadot_parachain_primitives::primitives::Sibling;
use polkadot_sdk::sp_runtime::traits::AccountIdConversion;
use polkadot_sdk::{
    frame_support::traits::tokens::{fungible, fungibles},
    sp_runtime, staging_xcm as xcm,
};
use xcm::latest::Location;

const ALICE: u32 = 1;
const BALANCE: u128 = 1_000_000_000_000_000; // 1000 WND

#[test]
fn test_register_native_asset_creates_foreign_asset_on_ah() {
    let parachain_location = Location::new(1, Parachain(2000));

    AssetHubWestend::execute_with(|| {
        assert!(
            !<AssetHubWestend as AssetHubWestendPallet>::ForeignAssets::asset_exists(
                parachain_location.clone()
            )
        );

        let sovereign_account: AccountId =
            Sibling::from(ParaId::from(2000)).into_account_truncating();

        assert_ok!(
            <AssetHubWestend as AssetHubWestendPallet>::Balances::force_set_balance(
                asset_hub_westend_runtime::RuntimeOrigin::root(),
                sovereign_account.into(),
                BALANCE,
            )
        );
    });

    CustomPara::execute_with(|| {
        assert_ok!(
            <CustomPara as CustomParaPallet>::XcmUtils::register_native_asset_on_ah(
                parachain_runtime::RuntimeOrigin::root()
            )
        );
    });

    AssetHubWestend::execute_with(|| {
        assert!(
            <AssetHubWestend as AssetHubWestendPallet>::ForeignAssets::asset_exists(
                parachain_location
            )
        );
    });
}
