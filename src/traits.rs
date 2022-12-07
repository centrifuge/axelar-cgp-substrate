//! Traits used by the Axelar pallet

// ----------------------------------------------------------------------------
// Module imports and re-exports
// ----------------------------------------------------------------------------

// Frame, system and frame primitives
use frame_support::weights::Weight;

// ----------------------------------------------------------------------------
// Traits declaration
// ----------------------------------------------------------------------------

/// Weight information for pallet extrinsics
///
/// Weights are calculated using runtime benchmarking features.
/// See [`benchmarking`] module for more information.
pub trait WeightInfo {
    fn temp() -> Weight;
}
