use crate::{
    AccountId, AllPalletsWithSystem, Balance, Balances, ForeignAssets, ParachainInfo,
    ParachainSystem, PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin, WeightToFee,
    XcmpQueue,
};

use polkadot_sdk::{
    staging_xcm as xcm, staging_xcm_builder as xcm_builder, staging_xcm_executor as xcm_executor, *,
};

use frame_support::{
    parameter_types,
    traits::{ConstU32, Contains, ContainsPair, Everything, EverythingBut, Nothing},
    weights::Weight,
};
use frame_system::EnsureRoot;
use pallet_xcm::XcmPassthrough;
use polkadot_parachain_primitives::primitives::Sibling;
use polkadot_runtime_common::impls::ToAuthor;
use polkadot_sdk::polkadot_sdk_frame::traits::ProcessMessageError;
use polkadot_sdk::{
    polkadot_sdk_frame::traits::Disabled,
    staging_xcm_builder::{DenyRecursively, DenyThenTry},
};
use sp_runtime::traits::TryConvertInto;
use xcm::latest::prelude::*;
use xcm_builder::{
    AccountId32Aliases, AllowExplicitUnpaidExecutionFrom, AllowTopLevelPaidExecutionFrom,
    DenyReserveTransferToRelayChain, EnsureXcmOrigin, FixedWeightBounds,
    FrameTransactionalProcessor, FungibleAdapter, FungiblesAdapter, IsConcrete, NoChecking,
    ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
    SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, StartsWith,
    TakeWeightCredit, TrailingSetTopicAsId, UsingComponents, WithComputedOrigin, WithUniqueTopic,
};
use xcm_executor::{
    traits::{DenyExecution, Identity},
    XcmExecutor,
};

pub const ASSET_HUB_PARA_ID: u32 = 1000;

parameter_types! {
    pub const HereLocation: Location = Location::here();
    pub const RelayLocation: Location = Location::parent();
    pub const RelayNetwork: Option<NetworkId> = None;
    pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
    // For the real deployment, it is recommended to set `RelayNetwork` according to the relay chain
    // and prepend `UniversalLocation` with `GlobalConsensus(RelayNetwork::get())`.
    pub UniversalLocation: InteriorLocation = Parachain(ParachainInfo::parachain_id().into()).into();
    // Optional: banned account on Asset Hub (AccountId32). Adjust as needed.
    // Any XCM originating from `Parent -> Parachain(1000) -> AccountId32(banned)` is rejected.
    pub const BannedAssetHubAccountId: [u8; 32] = [0u8; 32];
}

// Optional: reject XCM messages originating from a specific account on Asset Hub.
// This is a simple, explicit deny-list example for the assignment bonus.
pub struct DenyBannedAssetHubAccount;
impl DenyExecution for DenyBannedAssetHubAccount {
    fn deny_execution<RuntimeCall>(
        origin: &Location,
        _instructions: &mut [Instruction<RuntimeCall>],
        _max_weight: Weight,
        _properties: &mut xcm_executor::traits::Properties,
    ) -> Result<(), ProcessMessageError> {
        match origin.unpack() {
            (1, [Parachain(id), AccountId32 { id: account_id, .. }])
                if *id == ASSET_HUB_PARA_ID && *account_id == BannedAssetHubAccountId::get() =>
            {
                // Deny execution for this banned origin.
                Err(ProcessMessageError::Unsupported)
            }
            _ => Ok(()),
        }
    }
}

/// Type for specifying how a `Location` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
    // The parent (Relay-chain) origin converts to the parent `AccountId`.
    ParentIsPreset<AccountId>,
    // Sibling parachain origins convert to AccountId via the `ParaId::into`.
    SiblingParachainConvertsVia<Sibling, AccountId>,
    // Straight up local `AccountId32` origins just alias directly to `AccountId`.
    AccountId32Aliases<RelayNetwork, AccountId>,
);

/// Means for transacting assets on this chain.
pub type FungibleTransactor = FungibleAdapter<
    // Use this currency:
    Balances,
    // Use this currency when it is a fungible asset matching the given location or name:
    IsConcrete<HereLocation>,
    // Do a simple punn to convert an AccountId32 Location into a native chain account ID:
    LocationToAccountId,
    // Our chain's account ID type (we can't get away without mentioning it explicitly):
    AccountId,
    // We don't track any teleports.
    (),
>;

pub type ForeignAssetsAssetId = Location;
pub type ForeignAssetsConvertedConcreteId = xcm_builder::MatchedConvertedConcreteId<
    Location,
    Balance,
    EverythingBut<(
        // Here we rely on fact that something like this works:
        // assert!(Location::new(1,
        // [Parachain(100)]).starts_with(&Location::parent()));
        // assert!([Parachain(100)].into().starts_with(&Here));
        StartsWith<HereLocation>,
    )>,
    Identity,
    TryConvertInto,
>;

/// Means for transacting foreign assets from different global consensus.
pub type ForeignFungiblesTransactor = FungiblesAdapter<
    // Use this fungibles implementation:
    ForeignAssets,
    // Use this currency when it is a fungible asset matching the given location or name:
    ForeignAssetsConvertedConcreteId,
    // Convert an XCM Location into a local account id:
    LocationToAccountId,
    // Our chain's account ID type (we can't get away without mentioning it explicitly):
    AccountId,
    // We don't need to check teleports here.
    NoChecking,
    // The account to use for tracking teleports.
    CheckingAccount,
>;

/// Means for transacting assets on this chain.
pub type AssetTransactors = (FungibleTransactor, ForeignFungiblesTransactor);

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
    // Sovereign account converter; this attempts to derive an `AccountId` from the origin location
    // using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
    // foreign chains who want to have a local sovereign account on this chain which they control.
    SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
    // Native converter for Relay-chain (Parent) location; will convert to a `Relay` origin when
    // recognized.
    RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
    // Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
    // recognized.
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
    // Native signed account converter; this just converts an `AccountId32` origin into a normal
    // `RuntimeOrigin::Signed` origin of the same 32-byte value.
    SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
    // Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
    XcmPassthrough<RuntimeOrigin>,
);

parameter_types! {
    // One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
    pub UnitWeightCost: Weight = Weight::from_parts(1_000_000_000, 64 * 1024);
    pub const MaxInstructions: u32 = 100;
    pub const MaxAssetsIntoHolding: u32 = 64;
    pub CheckingAccount: AccountId = PolkadotXcm::check_account();
}

pub struct ParentOrParentsExecutivePlurality;
impl Contains<Location> for ParentOrParentsExecutivePlurality {
    fn contains(location: &Location) -> bool {
        matches!(
            location.unpack(),
            (1, [])
                | (
                    1,
                    [Plurality {
                        id: BodyId::Executive,
                        ..
                    }]
                )
        )
    }
}

pub type Barrier = TrailingSetTopicAsId<
    DenyThenTry<
        DenyRecursively<(DenyReserveTransferToRelayChain, DenyBannedAssetHubAccount)>,
        (
            TakeWeightCredit,
            WithComputedOrigin<
                (
                    AllowTopLevelPaidExecutionFrom<Everything>,
                    AllowExplicitUnpaidExecutionFrom<ParentOrParentsExecutivePlurality>,
                    // ^^^ Parent and its exec plurality get free execution
                ),
                UniversalLocation,
                ConstU32<8>,
            >,
        ),
    >,
>;

pub struct AssetHubForWnd;
impl ContainsPair<Asset, Location> for AssetHubForWnd {
    fn contains(asset: &Asset, location: &Location) -> bool {
        let is_asset_hub = match location.unpack() {
            (1, [Parachain(id)]) if *id == ASSET_HUB_PARA_ID => true,
            _ => false,
        };

        asset.id.0 == Location::parent() && is_asset_hub
    }
}

pub struct NativeAssetToAssetHub;
impl ContainsPair<Asset, Location> for NativeAssetToAssetHub {
    fn contains(asset: &Asset, location: &Location) -> bool {
        // Only allow teleports of the local native asset to Asset Hub.
        let is_asset_hub = match location.unpack() {
            (1, [Parachain(id)]) if *id == ASSET_HUB_PARA_ID => true,
            _ => false,
        };

        asset.id.0 == Location::here() && is_asset_hub
    }
}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
    type RuntimeCall = RuntimeCall;
    type XcmSender = XcmRouter;
    type XcmEventEmitter = PolkadotXcm;
    // How to withdraw and deposit an asset.
    type AssetTransactor = AssetTransactors;
    type OriginConverter = XcmOriginToTransactDispatchOrigin;
    type IsReserve = AssetHubForWnd;
    // Restrictive whitelist for teleports: only native asset -> Asset Hub.
    // This is required by the assignment (IsTeleporter was `()`), and keeps
    // teleports limited to a safe scope.
    type IsTeleporter = NativeAssetToAssetHub;
    type UniversalLocation = UniversalLocation;
    type Barrier = Barrier;
    // All instructions cost the same weight.
    // This is a testing configuration and should never be used in production.
    type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
    type Trader =
        UsingComponents<WeightToFee, HereLocation, AccountId, Balances, ToAuthor<Runtime>>;
    type ResponseHandler = PolkadotXcm;
    type AssetTrap = PolkadotXcm;
    type AssetClaims = PolkadotXcm;
    type SubscriptionService = PolkadotXcm;
    type PalletInstancesInfo = AllPalletsWithSystem;
    type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
    type AssetLocker = ();
    type AssetExchanger = ();
    type FeeManager = ();
    type MessageExporter = ();
    type UniversalAliases = Nothing;
    type CallDispatcher = RuntimeCall;
    type SafeCallFilter = Everything;
    type Aliasers = Nothing;
    type TransactionalProcessor = FrameTransactionalProcessor;
    type HrmpNewChannelOpenRequestHandler = ();
    type HrmpChannelAcceptedHandler = ();
    type HrmpChannelClosingHandler = ();
    type XcmRecorder = PolkadotXcm;
}

/// Converts a local signed origin into an XCM location. Forms the basis for local origins
/// sending/executing XCMs.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = WithUniqueTopic<(
    // Two routers - use UMP to communicate with the relay chain:
    cumulus_primitives_utility::ParentAsUmp<ParachainSystem, (), ()>,
    // ..and XCMP to communicate with the sibling chains.
    XcmpQueue,
)>;

type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;

impl pallet_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    type ExecuteXcmOrigin = ExecuteXcmOrigin;
    type XcmExecuteFilter = Everything;
    // ^ Disable dispatchable execute on the XCM pallet.
    // Needs to be `Everything` for local testing.
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type XcmTeleportFilter = Everything;
    type XcmReserveTransferFilter = Nothing;
    type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
    type UniversalLocation = UniversalLocation;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;

    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
    // ^ Override for AdvertisedXcmVersion default
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
    type Currency = Balances;
    type CurrencyMatcher = ();
    type TrustedLockers = ();
    type SovereignAccountOf = LocationToAccountId;
    type MaxLockers = ConstU32<8>;
    type WeightInfo = pallet_xcm::TestWeightInfo;
    type AdminOrigin = EnsureRoot<AccountId>;
    type MaxRemoteLockConsumers = ConstU32<0>;
    type RemoteLockConsumerIdentifier = ();
    // Aliasing is disabled: xcm_executor::Config::Aliasers is set to `Nothing`.
    type AuthorizedAliasConsideration = Disabled;
}

impl cumulus_pallet_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

/// Configure the XCM utils pallet.
impl pallet_xcm_utils::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Xcm = PolkadotXcm;
    type WeightInfo = pallet_xcm_utils::weights::SubstrateWeight<Runtime>;
}
