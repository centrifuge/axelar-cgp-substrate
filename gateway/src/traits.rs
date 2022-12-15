//! Traits used by the Axelar pallet

// ----------------------------------------------------------------------------
// Module imports and re-exports
// ----------------------------------------------------------------------------

// Frame, system and frame primitives
use frame_support::weights::{constants::RocksDbWeight, Weight};

// ----------------------------------------------------------------------------
// Traits declaration
// ----------------------------------------------------------------------------

/// Weight information for pallet extrinsics
///
/// Weights are calculated using runtime benchmarking features.
/// See [`benchmarking`] module for more information.
pub trait WeightInfo {
    fn transfer_operatorship() -> Weight;
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn transfer_operatorship() -> Weight {
        (Weight::from_ref_time(65_453_000))
            .saturating_add(RocksDbWeight::get().reads(3_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
}
