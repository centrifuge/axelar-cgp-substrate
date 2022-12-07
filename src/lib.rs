//! # Axelar CGP
//!
//! This pallet implements the Axelar Cross-Chain Gateway Protocol

// Ensure we're `no_std` when compiling for WebAssembly.
#![cfg_attr(not(feature = "std"), no_std)]

// ----------------------------------------------------------------------------
// Module imports and re-exports
// ----------------------------------------------------------------------------

// Pallet traits declaration
pub mod traits;

// Re-export pallet components in crate namespace (for runtime construction)
pub use pallet::*;

use crate::traits::WeightInfo;

// ----------------------------------------------------------------------------
// Type aliases
// ----------------------------------------------------------------------------

// ----------------------------------------------------------------------------
// Pallet module
// ----------------------------------------------------------------------------

// Axelar pallet module
//
// The name of the pallet is provided by `construct_runtime` and is used as
// the unique identifier for the pallet's storage. It is not defined in the
// pallet itself.
#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::ArithmeticError;

    use super::*;

    // Axelar pallet type declaration.
    //
    // This structure is a placeholder for traits and functions implementation
    // for the pallet.
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // ------------------------------------------------------------------------
    // Pallet configuration
    // ------------------------------------------------------------------------

    /// Axelar pallet's configuration trait.
    ///
    /// Associated types and constants are declared in this trait. If the pallet
    /// depends on other super-traits, the latter must be added to this trait,
    /// Note that [`frame_system::Config`] must always be included.
    #[pallet::config]
    pub trait Config: frame_system::Config {

        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Weight information for extrinsics in this pallet
        type WeightInfo: WeightInfo;
    }

    // ------------------------------------------------------------------------
    // Pallet events
    // ------------------------------------------------------------------------

    // The macro generates event metadata and derive Clone, Debug, Eq, PartialEq and Codec
    #[pallet::event]
    // The macro generates a function on Pallet to deposit an event
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        OperatorshipTransferred,
    }


    // ------------------------------------------------------------------------
    // Pallet storage
    // ------------------------------------------------------------------------

    #[pallet::storage]
    #[pallet::getter(fn current_epoch)]
    pub(super) type CurrentEpoch<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn hash_for_epoch)]
    pub(super) type HashForEpoch<T: Config> = StorageMap<_, Blake2_128Concat, u64, T::Hash, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn epoch_for_hash)]
    pub(super) type EpochForHash<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, u64, ValueQuery>;

    // ------------------------------------------------------------------------
    // Pallet errors
    // ------------------------------------------------------------------------

    #[pallet::error]
    pub enum Error<T> {
    }

    // ------------------------------------------------------------------------
    // Pallet dispatchable functions
    // ------------------------------------------------------------------------

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // TODO: Should be internal function only callable from an approved Gateway call
        #[pallet::weight(<T as pallet::Config>::WeightInfo::transfer_operatorship())]
        pub fn transfer_operatorship(
            _origin: OriginFor<T>
        ) -> DispatchResult {
            // Add Authorize filter according to the execute strategy
            Self::validate_operatorship()?;

            let new_operator_hash = T::Hash::default();
            let epoch = <CurrentEpoch<T>>::get().checked_add(1).ok_or(ArithmeticError::Overflow)?;
            <CurrentEpoch<T>>::set(epoch);
            <HashForEpoch<T>>::set(epoch, new_operator_hash);
            <EpochForHash<T>>::set(new_operator_hash, epoch);

            Self::deposit_event(Event::OperatorshipTransferred);
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn validate_operatorship() -> Result<bool, DispatchError> {
            Ok(true)
        }
    }

} // end of 'pallet' module

