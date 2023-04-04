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

pub trait WeightInfo {
	fn sample_final_call() -> Weight;
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn sample_final_call() -> Weight {
		Zero::zero()
	}
}