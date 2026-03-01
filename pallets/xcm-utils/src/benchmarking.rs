#![cfg(feature = "runtime-benchmarks")]

use crate::Config;
use frame::benchmarking::prelude::*;

/// Pallet we're benchmarking.
pub struct Pallet<T: Config>(crate::Pallet<T>);

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register_native_asset_on_ah() -> Result<(), BenchmarkError> {
        #[extrinsic_call]
        _(RawOrigin::Root);

        Ok(())
    }

    #[benchmark]
    fn transfer_to_ah() -> Result<(), BenchmarkError> {
        let caller: T::AccountId = whitelisted_caller();
        let amount_native: u128 = 1;
        let amount_wnd: u128 = 1;
        let beneficiary: [u8; 32] = [1u8; 32];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), amount_native, amount_wnd, beneficiary);

        Ok(())
    }
}
