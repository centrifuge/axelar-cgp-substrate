//! Traits used by the Axelar pallet

// ----------------------------------------------------------------------------
// Module imports and re-exports
// ----------------------------------------------------------------------------

use frame_support::PalletId;
// Frame, system and frame primitives
use frame_support::weights::{constants::RocksDbWeight, Weight};

// ----------------------------------------------------------------------------
// Traits declaration
// ----------------------------------------------------------------------------

pub const GATEWAY_PALLET_ID: PalletId = PalletId(*b"axgteway");

/// Weight information for pallet extrinsics
///
/// Weights are calculated using runtime benchmarking features.
/// See [`benchmarking`] module for more information.
pub trait WeightInfo {
    fn execute() -> Weight;
    fn transfer_operatorship(c: u32) -> Weight;
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn execute() -> Weight {
        (Weight::from_ref_time(65_453_000))
            .saturating_add(RocksDbWeight::get().reads(3_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    fn transfer_operatorship(c: u32) -> Weight {
        // Minimum execution time: 14_470 nanoseconds.
        Weight::from_ref_time(17_443_346 as u64)
            // Standard Error: 2_037
            .saturating_add(Weight::from_ref_time(3_510_555 as u64).saturating_mul(c as u64))
    }
}
