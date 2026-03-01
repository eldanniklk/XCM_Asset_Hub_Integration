#![allow(clippy::unnecessary_cast)]

use core::marker::PhantomData;
// Use the FRAME umbrella crate re-exports to avoid direct dependency on frame_support.
use frame::deps::{frame_support, frame_system};
use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// Weight functions needed for `pallet_xcm_utils`.
pub trait WeightInfo {
    fn register_native_asset_on_ah() -> Weight;
    fn transfer_to_ah() -> Weight;
    // Optional: setup pool on Asset Hub (create pool + add liquidity).
    fn setup_pool_on_ah() -> Weight;
}

/// Default weights for this pallet.
///
/// NOTE: These are placeholder weights. Run runtime benchmarks and replace
/// these values before production use.
pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn register_native_asset_on_ah() -> Weight {
        Weight::from_parts(10_000, 0).saturating_add(T::DbWeight::get().writes(1))
    }

    fn transfer_to_ah() -> Weight {
        Weight::from_parts(10_000, 0).saturating_add(T::DbWeight::get().reads_writes(1, 1))
    }

    fn setup_pool_on_ah() -> Weight {
        Weight::from_parts(10_000, 0).saturating_add(T::DbWeight::get().reads_writes(1, 1))
    }
}

// For tests/benchmarks without a runtime.
impl WeightInfo for () {
    fn register_native_asset_on_ah() -> Weight {
        Weight::from_parts(10_000, 0).saturating_add(RocksDbWeight::get().writes(1))
    }

    fn transfer_to_ah() -> Weight {
        Weight::from_parts(10_000, 0).saturating_add(RocksDbWeight::get().reads_writes(1, 1))
    }

    fn setup_pool_on_ah() -> Weight {
        Weight::from_parts(10_000, 0).saturating_add(RocksDbWeight::get().reads_writes(1, 1))
    }
}
