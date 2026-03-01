//! Module for writing the actual integration tests of your solution.

use polkadot_sdk::xcm_emulator::assert_ok;

use super::prelude::*;
use crate::tests::xcm::opaque::latest::Junction::Parachain;
use parachain_runtime::RuntimeOrigin;
use polkadot_sdk::cumulus_primitives_core::ParaId;
use polkadot_sdk::emulated_integration_tests_common::impls::Inspect;
use polkadot_sdk::frame_support::traits::fungibles::Mutate;
use polkadot_sdk::polkadot_parachain_primitives::primitives::Sibling;
use polkadot_sdk::sp_keyring::Sr25519Keyring;
use polkadot_sdk::sp_runtime::traits::AccountIdConversion;
use polkadot_sdk::{
    frame_support::traits::tokens::{fungible, fungibles},
    sp_runtime, staging_xcm as xcm,
};
use xcm::latest::Location;

const ALICE: u32 = 1;
const BOB: u32 = 2;
const BALANCE: u128 = 1_000_000_000_000_000; // 1000 WND

#[test]
fn test_register_native_asset_creates_foreign_asset_on_ah_works() {
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

    AssetHubWestend::execute_with(|| {
        // Check fee refund
        let sovereign_account: AccountId =
            Sibling::from(ParaId::from(2000)).into_account_truncating();

        let final_balance =
            <AssetHubWestend as AssetHubWestendPallet>::Balances::free_balance(&sovereign_account);

        // The balance should be less than initial (some fees were consumed)
        // but should have received a refund (so not all 1 WND should be consumed)
        let consumed_fees = BALANCE - final_balance;
        let one_wnd = 1_000_000_000_000u128; // 1 WND

        // Verify that less than the full 1 WND was consumed (meaning refund occurred)
        assert!(consumed_fees < one_wnd);

        // Verify that some fees were actually consumed (operation wasn't free)
        assert!(consumed_fees > 0);
    });
}

#[test]
fn test_transfer_to_ah_works() {
    use polkadot_sdk::frame_support::traits::fungible::Inspect;

    let sovereign_account: AccountId = Sibling::from(ParaId::from(2000)).into_account_truncating();
    let parachain_location = Location::new(1, Parachain(2000));
    let relay_location = Location::parent();
    let alice = Sr25519Keyring::Alice.to_account_id();
    let bob = Sr25519Keyring::Bob.to_account_id();
    let balance_to_transfer = BALANCE / 10;

    let bob_balance = AssetHubWestend::execute_with(|| {
        // Set the balance of the sovereign account on Asset Hub.
        assert_ok!(
            <AssetHubWestend as AssetHubWestendPallet>::Balances::force_set_balance(
                asset_hub_westend_runtime::RuntimeOrigin::root(),
                sovereign_account.into(),
                BALANCE,
            )
        );

        <AssetHubWestend as AssetHubWestendPallet>::Balances::total_balance(&bob)
    });

    CustomPara::execute_with(|| {
        // Register the native asset first.
        assert_ok!(
            <CustomPara as CustomParaPallet>::XcmUtils::register_native_asset_on_ah(
                parachain_runtime::RuntimeOrigin::root()
            )
        );

        // Set Alice's balance in the native asset.
        assert_ok!(
            <CustomPara as CustomParaPallet>::Balances::force_set_balance(
                parachain_runtime::RuntimeOrigin::root(),
                alice.clone().into(),
                balance_to_transfer,
            )
        );

        // Check Alice has no balance in WND
        assert_eq!(
            <CustomPara as CustomParaPallet>::ForeignAssets::total_balance(
                relay_location.clone(),
                (&alice).into()
            ),
            0
        );

        // Set Alice's balance in WND.
        assert_ok!(<CustomPara as CustomParaPallet>::ForeignAssets::mint_into(
            relay_location.clone(),
            (&alice.clone()).into(),
            BALANCE,
        ));
        assert_eq!(
            <CustomPara as CustomParaPallet>::ForeignAssets::total_balance(
                relay_location.clone(),
                (&alice).into()
            ),
            BALANCE
        );

        // Send from Alice to Bob
        assert_ok!(<CustomPara as CustomParaPallet>::XcmUtils::transfer_to_ah(
            parachain_runtime::RuntimeOrigin::signed(alice.clone()),
            balance_to_transfer,
            balance_to_transfer,
            bob.clone().into(),
        ));
    });

    AssetHubWestend::execute_with(|| {
        // Check Bob's balance is non-zero.
        assert_eq!(
            <AssetHubWestend as AssetHubWestendPallet>::ForeignAssets::total_balance(
                parachain_location,
                (&bob.clone()).into(),
            ),
            balance_to_transfer
        );

        assert_eq!(
            <AssetHubWestend as AssetHubWestendPallet>::Balances::total_balance(&bob),
            bob_balance + balance_to_transfer
        );
    });
}
