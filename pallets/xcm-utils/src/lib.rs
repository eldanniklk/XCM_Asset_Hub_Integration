#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use pallet::*;

#[frame::pallet]
pub mod pallet {
	use polkadot_sdk::{staging_xcm as xcm, staging_xcm_builder::{ExecuteController, SendController}, sp_runtime};
	use xcm::prelude::*;
	use frame::prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config +  {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		// An interface to access XCM `execute` and `send`.
		type Xcm: ExecuteController<OriginFor<Self>, Self::RuntimeCall> + SendController<OriginFor<Self>>;
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
		create { id: xcm::v5::Location, admin: sp_runtime::MultiAddress<sp_runtime::AccountId32, ()>, min_balance: u128 }
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Registers the native asset on Asset Hub.
		///
		/// Must be called by root.
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn register_native_asset_on_ah(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;

			// TODO.

			Self::deposit_event(Event::<T>::Example);

			Ok(())
		}

		/// Sends a certain `amount_native` of the native asset and `amount_wnd` of WND to Asset Hub.
		///
		/// The native asset has to be registered first via `register_native_asset_on_ah`.
		/// Can be called by any signed origin.
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
		pub fn transfer_to_ah(origin: OriginFor<T>, _amount_native: u128, _amount_wnd: u128, _beneficiary: [u8; 32]) -> DispatchResult {
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

			let bytes: [u8; 32] = sp_core::bytes::from_hex(hex)
				.unwrap()
				.try_into()
				.unwrap();
			sp_runtime::AccountId32::new(bytes)
		}
	}
}
