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

use frame_support::weights::Weight;
use sp_runtime::traits::Zero;

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
		Zero::zero()
	}

	fn transfer_operatorship(c: u32) -> Weight {
		Zero::zero()
	}

	fn approve_contract_call() -> Weight {
		Zero::zero()
	}

	fn forward_approved_call() -> Weight {
		Zero::zero()
	}

	fn call_contract() -> Weight {
		Zero::zero()
	}
}
