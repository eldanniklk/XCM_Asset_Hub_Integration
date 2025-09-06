mod chains;
mod network;
#[cfg(test)]
mod tests;

pub mod prelude {
    use super::*;
    use polkadot_sdk::*;

    pub use network::{
        AssetHubWestendPara as AssetHubWestend,
        AssetHubWestendParaReceiver as AssetHubWestendReceiver,
        AssetHubWestendParaSender as AssetHubWestendSender, CustomPara, CustomParaReceiver,
        CustomParaSender, WestendRelay as Westend, WestendRelayReceiver as WestendReceiver,
        WestendRelaySender as WestendSender,
    };

    pub use chains::{AssetHubWestendPallet, CustomParaPallet, WestendPallet};

    pub use xcm_emulator::{assert_expected_events, Chain, Parachain, TestExt};

    pub use sp_runtime::AccountId32 as AccountId;

    pub use parachain_runtime::{CENTS as PARA_CENTS, UNITS as PARA_UNITS};
    pub use westend_runtime_constants::currency::{CENTS as WND_CENTS, UNITS as WND_UNITS};
}
