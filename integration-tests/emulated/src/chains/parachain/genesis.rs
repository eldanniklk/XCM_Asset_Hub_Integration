use polkadot_sdk::{staging_xcm as xcm, *};
use polkadot_sdk::staging_parachain_info as parachain_info;

use emulated_integration_tests_common::build_genesis_storage;
use frame_support::parameter_types;
use sp_runtime::Storage;
use sp_keyring::Sr25519Keyring as Keyring;
use xcm::prelude::*;

pub const ED: u128 = 100_000_000;

parameter_types! {
	pub AssetOwner: sp_runtime::AccountId32 = Keyring::Alice.to_account_id();
}

pub fn genesis() -> Storage {
	let genesis_config = parachain_runtime::RuntimeGenesisConfig {
		parachain_info: parachain_info::GenesisConfig {
			parachain_id: 2000.into(),
			..Default::default()
		},
		foreign_assets: parachain_runtime::ForeignAssetsConfig {
			assets: vec![
				// Relay chain asset.
				(Location::parent(), AssetOwner::get(), false, ED)
			],
			..Default::default()
		},
		..Default::default()
	};

	build_genesis_storage(
		&genesis_config,
		parachain_runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
	)
}
