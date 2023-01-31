//! Traits used by the Axelar pallet

// ----------------------------------------------------------------------------
// Module imports and re-exports
// ----------------------------------------------------------------------------

use frame_support::PalletId;
// Frame, system and frame primitives
use frame_support::weights::Weight;

// ----------------------------------------------------------------------------
// Traits declaration
// ----------------------------------------------------------------------------

pub const GATEWAY_PALLET_ID: PalletId = PalletId(*b"axgteway");

/// Weight information for pallet extrinsics
///
/// Weights are calculated using runtime benchmarking features.
/// See [`benchmarking`] module for more information.
pub trait WeightInfo {
    fn execute(c: u32) -> Weight;
    fn transfer_operatorship(c: u32) -> Weight;
    fn approve_contract_call() -> Weight;
    fn execute_approved_call() -> Weight;
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn execute(c: u32) -> Weight {
        // Minimum execution time: 13_885 nanoseconds.
        Weight::from_ref_time(20_147_978 as u64)
            // Standard Error: 2_232
            .saturating_add(Weight::from_ref_time(3_516_969 as u64).saturating_mul(c as u64))
    }
    fn transfer_operatorship(c: u32) -> Weight {
        // Minimum execution time: 14_470 nanoseconds.
        Weight::from_ref_time(17_443_346 as u64)
            // Standard Error: 2_037
            .saturating_add(Weight::from_ref_time(3_510_555 as u64).saturating_mul(c as u64))
    }
    fn approve_contract_call() -> Weight {
        Weight::from_ref_time(17_443_346 as u64)
    }
    fn execute_approved_call() -> Weight {
        Weight::from_ref_time(17_443_346 as u64)
    }
}
