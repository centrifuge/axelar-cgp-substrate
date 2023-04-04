// Copyright 2023 Centrifuge Foundation (centrifuge.io).
//
// This file is part of the axelar-cgp-substrate project.
// Axelar-cgp-substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version (see http://www.gnu.org/licenses).
// Centrifuge is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

//! # Sample Receiver Pallet
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
use crate::traits::WeightInfo;
use codec::{Decode, Encode, MaxEncodedLen};
pub use pallet::*;
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum RawOrigin {
    Dummy,
}

// ----------------------------------------------------------------------------
// Constants
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
    use pallet_xcm::ensure_xcm;
    use sp_core::{keccak_256, H256};
    use xcm::latest::MultiLocation;
    use xcm::prelude::{Junction, X2};

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
    pub trait Config:
        frame_system::Config<RuntimeOrigin = <Self as Config>::RuntimeOrigin>
        + pallet_xcm::Config<RuntimeOrigin = <Self as frame_system::Config>::RuntimeOrigin>
    {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type RuntimeOrigin: From<RawOrigin>
            + Into<Result<RawOrigin, <Self as Config>::RuntimeOrigin>>
            + Into<
                Result<
                    frame_system::RawOrigin<<Self as frame_system::Config>::AccountId>,
                    <Self as Config>::RuntimeOrigin,
                >,
            > + Into<Result<pallet_xcm::Origin, <Self as Config>::RuntimeOrigin>>;

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
        SampleFinalCall {
            sender: String,
            proxy_chain: String,
            data: H256,
        },
    }

    #[pallet::origin]
    pub type Origin = RawOrigin;

    // ------------------------------------------------------------------------
    // Pallet storage
    // ------------------------------------------------------------------------

    // ------------------------------------------------------------------------
    // Pallet errors
    // ------------------------------------------------------------------------

    #[pallet::error]
    pub enum Error<T> {
        InvalidOrigin,
    }

    // ------------------------------------------------------------------------
    // Pallet dispatchable functions
    // ------------------------------------------------------------------------

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// This call is an example of how a receiving pallet could deal with the multilocation hierarchy
        /// so can authorize the caller
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::sample_final_call())]
        pub fn sample_final_call(origin: OriginFor<T>, data: Vec<u8>) -> DispatchResult {
            let origin_location = ensure_xcm(origin)?;

            // Authorize call
            let origin_data = match origin_location {
                MultiLocation {
                    parents: 1,
                    interior:
                        X2(
                            Junction::Parachain(id),
                            Junction::AccountKey20 {
                                network: _,
                                key: acc_id,
                            },
                        ),
                } => Ok((id, acc_id)),
                _ => Err(Error::<T>::InvalidOrigin),
            }?;

            Self::deposit_event(Event::SampleFinalCall {
                proxy_chain: origin_data.0.to_string(),
                sender: hex::encode(origin_data.1.as_slice()),
                data: H256::from_slice(keccak_256(&data).as_slice()),
            });

            Ok(())
        }
    }
}
// end of 'pallet' module
