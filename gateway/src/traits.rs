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

//! Traits used by the Axelar pallet

// ----------------------------------------------------------------------------
// Module imports and re-exports
// ----------------------------------------------------------------------------

use std::{marker::PhantomData, str::FromStr};

use codec::Decode;
use frame_support::{dispatch::DispatchResult, traits::EnsureOrigin, weights::Weight};
use pallet_xcm::ensure_xcm;
use sp_core::{bounded::WeakBoundedVec, ConstU32, Get, H160};
use sp_runtime::{traits::Dispatchable, DispatchError};
use xcm::{
	latest::{prelude::*, Xcm},
	prelude::DescendOrigin,
};

// Frame, system and frame primitives
use crate::Error::ErrorForwarding;
use crate::{pallet, utils::vec_to_fixed_array, Config, MultiAddress, RawOrigin};

// ----------------------------------------------------------------------------
// Traits declaration
// ----------------------------------------------------------------------------

/// Weight information for pallet extrinsics
///
/// Weights are calculated using runtime benchmarking features.
/// See [`benchmarking`] module for more information.
pub trait WeightInfo {
	fn execute(c: u32) -> Weight;
	fn transfer_operatorship(c: u32) -> Weight;
	fn approve_contract_call() -> Weight;
	fn forward_approved_call() -> Weight;
	fn call_contract() -> Weight;
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

	fn forward_approved_call() -> Weight {
		Weight::from_ref_time(17_443_346 as u64)
	}

	fn call_contract() -> Weight {
		Weight::from_ref_time(17_443_346 as u64)
	}
}

pub trait CallForwarder<T: pallet::Config> {
	fn is_local() -> bool;
	fn do_forward(
		source_chain: String,
		source_address: String,
		dest: u32,
		call: Vec<u8>,
	) -> DispatchResult;
}

/// Local Forwarder Default Implementation
/// Just an example of how a local forwarder/executor could look like
pub struct LocalCallForwarder;
impl<T: Config> CallForwarder<T> for LocalCallForwarder {
	fn is_local() -> bool {
		true
	}

	fn do_forward(
		source_chain: String,
		source_address: String,
		_dest: u32,
		call: Vec<u8>,
	) -> DispatchResult {
		match <T as Config>::RuntimeCall::decode(&mut &call[..]) {
			Ok(final_call) => {
				// Add all origin types supported based on the source chains
				let source_origin = match source_chain.as_str() {
					"ethereum" => {
						let addr = H160::from_str(source_address.as_str())
							.map_err(|_| DispatchError::BadOrigin)?;
						RawOrigin::BridgeOnBehalfOf(
							vec_to_fixed_array(source_chain.into_bytes()),
							MultiAddress::Address20(addr.to_fixed_bytes()),
						)
					}
					_ => return Err(DispatchError::BadOrigin),
				};

				final_call
					.dispatch(source_origin.into())
					.map(|_| ())
					.map_err(|e| e.error)?;
				Ok(())
			}
			Err(_) => Err(DispatchError::CannotLookup),
		}
	}
}

/// XCM Forwarder Implementation
pub struct RemoteCallForwarder<XcmSender>(PhantomData<XcmSender>);
impl<T: Config, XcmSender: SendXcm> CallForwarder<T> for RemoteCallForwarder<XcmSender> {
	fn is_local() -> bool {
		false
	}

	fn do_forward(
		source_chain: String,
		source_address: String,
		dest: u32,
		call: Vec<u8>,
	) -> DispatchResult {
		// TODO: xcm v2 - review
		// Add all origin types supported based on the source chains
		let origin_junction = match source_chain.as_str() {
			// Named conversion can fail if source_chain is longer than 32b, revisit this
			// change to v3 NetworkId::Ethereum(chainId) or ByGenesis
			"ethereum" => {
				let addr = H160::from_str(source_address.as_str()).unwrap();
				Junction::AccountKey20 {
					network: NetworkId::Named(WeakBoundedVec::<u8, ConstU32<32>>::force_from(
						source_chain.into_bytes(),
						None,
					)),
					key: addr.to_fixed_bytes(),
				}
			}
			_ => return Err(DispatchError::BadOrigin),
		};

		// TODO: review fee setup
		let fee_asset = MultiAsset {
			id: Concrete(MultiLocation {
				parents: 1,
				interior: Junctions::Here,
			}),
			fun: Fungible(8_000_000_000),
		};

		let transact_message = Xcm(vec![
			WithdrawAsset(fee_asset.clone().into()),
			// TODO: review unlimited weight
			BuyExecution {
				fees: fee_asset.into(),
				weight_limit: WeightLimit::Unlimited,
			},
			// use xcm v3 when ready
			// UniversalOrigin(GlobalConsensus(NetworkId::Ethereum(id)))
			DescendOrigin(X1(origin_junction)),
			// TODO: review weight
			Transact {
				origin_type: OriginKind::Xcm,
				require_weight_at_most: 8_000_000_000,
				call: call.into(),
			},
		]);

		let dest_multi = MultiLocation {
			parents: 1,
			interior: X1(Parachain(dest)),
		};

		XcmSender::send_xcm(dest_multi, transact_message).map_err(|_| ErrorForwarding::<T>)?;

		Ok(())
	}
}

pub struct EnsureXcm;
impl<O: Into<Result<pallet_xcm::Origin, O>> + From<pallet_xcm::Origin> + Clone> EnsureOrigin<O>
	for EnsureXcm
{
	type Success = u32;

	fn try_origin(o: O) -> Result<Self::Success, O> {
		let location = ensure_xcm(o.clone()).map_err(|_| o.clone())?;

		match location {
			MultiLocation {
				parents: 1,
				interior: X1(Junction::Parachain(id)),
			} => Ok(id),
			_ => Err(o),
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> O {
		O::from(pallet_xcm::Origin::Xcm(MultiLocation {
			parents: 1,
			interior: Junctions::Here,
		}))
	}
}

pub struct EnsureLocal<ParaId>(PhantomData<ParaId>);
impl<O, ParaId: Get<u32>> EnsureOrigin<O> for EnsureLocal<ParaId> {
	type Success = u32;

	fn try_origin(_o: O) -> Result<Self::Success, O> {
		Ok(ParaId::get())
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> O {
		unimplemented!()
	}
}
