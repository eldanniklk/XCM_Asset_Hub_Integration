#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use pallet::*;

#[frame::pallet]
pub mod pallet {
    use alloc::boxed::Box;
    use alloc::vec;
    use frame::prelude::*;
    use polkadot_sdk::{
        sp_runtime, staging_xcm as xcm,
        staging_xcm_builder::{ExecuteController, SendController},
    };
    use xcm::prelude::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        // An interface to access XCM `execute` and `send`.
        type Xcm: ExecuteController<OriginFor<Self>, Self::RuntimeCall>
            + SendController<OriginFor<Self>>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Example,
    }

    #[pallet::error]
    pub enum Error<T> {
        Example,
    }

    // Provided to be able to encode the call for Asset Hub.
    #[derive(Encode)]
    #[allow(dead_code)]
    enum AssetHubWestendRuntimeCall {
        #[codec(index = 53)]
        ForeignAssets(ForeignAssetsCall),
    }

    #[derive(Encode)]
    #[allow(dead_code)]
    #[allow(non_camel_case_types)]
    enum ForeignAssetsCall {
        #[codec(index = 0)]
        create {
            id: xcm::v5::Location,
            admin: sp_runtime::MultiAddress<sp_runtime::AccountId32, ()>,
            min_balance: u128,
        },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Registers the native asset on Asset Hub.
        ///
        /// Must be called by root.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn register_native_asset_on_ah(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin.clone())?;

            let sov_account_on_ah: sp_runtime::AccountId32 = Self::public_key_to_account_id(
                "0x7369626cd0070000000000000000000000000000000000000000000000000000",
            );
            let sov_account_as_multiaddress: sp_runtime::MultiAddress<sp_runtime::AccountId32, ()> =
                sov_account_on_ah.clone().into();

            let origin_kind = OriginKind::Xcm;

            let fallback_max_weight = Weight::from_parts(10_000, 0);

            let call = AssetHubWestendRuntimeCall::ForeignAssets(ForeignAssetsCall::create {
                // The location of your chain from the perspective of asset hub.
                id: Location::new(1, [Parachain(2000)]),
                admin: sov_account_as_multiaddress.clone(),
                min_balance: 10_000u128,
            })
            .encode();

            let destination = Location::new(1, [Parachain(1000)]);
            const CENTS: u128 = 10_000_000_000; // 10^10
            let fees_amount = 10 * CENTS;
            let fees_assets: Asset = (Here, fees_amount.clone()).into();

            let refund_account = Location::new(
                0,
                [AccountId32 {
                    network: None,
                    id: sov_account_on_ah.clone().into(),
                }],
            );

            let xcm = Xcm::<T::RuntimeCall>(vec![
                // Pay fees
                WithdrawAsset(Assets::from(fees_assets.clone())),
                BuyExecution {
                    fees: fees_assets,
                    weight_limit: Unlimited,
                },
                // Perform the registration.
                Transact {
                    origin_kind: origin_kind,
                    fallback_max_weight: None,
                    call: call.encode().into(),
                },
                RefundSurplus,
                DepositAsset {
                    assets: All.into(),
                    beneficiary: refund_account,
                },
            ]);

            T::Xcm::send(
                origin,
                Box::new(VersionedLocation::from(destination)),
                Box::new(VersionedXcm::V5(xcm.into())),
            )?;

            Self::deposit_event(Event::<T>::Example);

            Ok(())
        }

        /// Sends a certain `amount_native` of the native asset and `amount_wnd` of WND to Asset Hub.
        ///
        /// The native asset has to be registered first via `register_native_asset_on_ah`.
        /// Can be called by any signed origin.
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
        pub fn transfer_to_ah(
            origin: OriginFor<T>,
            _amount_native: u128,
            _amount_wnd: u128,
            _beneficiary: [u8; 32],
        ) -> DispatchResult {
            let _who = ensure_signed(origin);

            // TODO.

            ensure!(false, Error::<T>::Example);

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        #[allow(dead_code)]
        /// Helper function you may or may not need to use to turn a hex string into the `AccountId` type.
        pub(crate) fn public_key_to_account_id(hex: &str) -> polkadot_sdk::sp_runtime::AccountId32 {
            use polkadot_sdk::{sp_core, sp_runtime};

            let bytes: [u8; 32] = sp_core::bytes::from_hex(hex).unwrap().try_into().unwrap();
            sp_runtime::AccountId32::new(bytes)
        }
    }
}
