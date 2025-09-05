mod genesis;

use polkadot_sdk::*;
use polkadot_sdk::staging_xcm as xcm;

use emulated_integration_tests_common::*;
use xcm_emulator::*;

decl_test_parachains! {
	pub struct Custom {
		genesis = genesis::genesis(),
		on_init = {},
		runtime = parachain_runtime,
		core = {
			XcmpMessageHandler: parachain_runtime::XcmpQueue,
			LocationToAccountId: parachain_runtime::configs::xcm_config::LocationToAccountId,
			ParachainInfo: parachain_runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
		},
		pallets = {
			System: parachain_runtime::System,
			Balances: parachain_runtime::Balances,
			ForeignAssets: parachain_runtime::ForeignAssets,
			PolkadotXcm: parachain_runtime::PolkadotXcm,
			XcmUtils: parachain_runtime::XcmUtils,
		}
	}
}

impl_foreign_assets_helpers_for_parachain!(Custom, xcm::v5::Location);
