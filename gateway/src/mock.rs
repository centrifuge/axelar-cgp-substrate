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

use std::marker::PhantomData;

use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU32, ConstU64, Everything, Nothing, OriginTrait},
};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup};
use xcm::{latest::prelude::*, opaque::latest::NetworkId};
use xcm_builder::{EnsureXcmOrigin, FixedWeightBounds, LocationInverter};

use crate::{
	self as pallet_axelar_cgp,
	traits::{EnsureXcm, LocalCallForwarder},
	Config,
};

pub type AccountId = u64;
pub type BlockNumber = u64;

impl frame_system::Config for Runtime {
	type AccountData = ();
	type AccountId = AccountId;
	type BaseCallFilter = Everything;
	type BlockHashCount = ConstU64<250>;
	type BlockLength = ();
	type BlockNumber = BlockNumber;
	type BlockWeights = ();
	type DbWeight = ();
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type Header = Header;
	type Index = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaxConsumers = ConstU32<16>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SS58Prefix = ();
	type SystemWeightInfo = ();
	type Version = ();
}

impl parachain_info::Config for Runtime {}

parameter_types! {
	pub const ChainId: u16 = 36;
	pub const RelayNetwork: NetworkId = NetworkId::Kusama;
	pub const UnitWeightCost: u64 = 10;
	pub MaxInstructions: u32 = 100;
	pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}

pub struct MockConversion<RuntimeOrigin, AccountId>(PhantomData<(RuntimeOrigin, AccountId)>);
impl<RuntimeOrigin: OriginTrait + Clone, AccountId: Into<u64>>
	xcm_executor::traits::Convert<RuntimeOrigin, MultiLocation>
	for MockConversion<RuntimeOrigin, AccountId>
where
	RuntimeOrigin::PalletsOrigin: From<frame_system::RawOrigin<AccountId>>
		+ TryInto<frame_system::RawOrigin<AccountId>, Error = RuntimeOrigin::PalletsOrigin>,
{
	fn convert(_o: RuntimeOrigin) -> Result<MultiLocation, RuntimeOrigin> {
		Ok(Junction::Parachain(1).into())
	}
}

pub type LocalOriginToLocation = MockConversion<RuntimeOrigin, AccountId>;

impl pallet_xcm::Config for Runtime {
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type LocationInverter = LocationInverter<Ancestry>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type XcmExecuteFilter = Everything;
	type XcmExecutor = ();
	type XcmReserveTransferFilter = Everything;
	type XcmRouter = ();
	type XcmTeleportFilter = Nothing;

	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
}

// parameter_types! {
//     pub LocalParaId: u32 = ParachainInfo::parachain_id().into();
// }

impl Config for Runtime {
	type ApprovedCallForwarder = LocalCallForwarder;
	type ChainId = ChainId;
	type EnsureCallOrigin = EnsureXcm;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type WeightInfo = ();
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		ParachainInfo: parachain_info::{Pallet, Storage, Config},
		PolkadotXcm: pallet_xcm::{Pallet, Call, Config, Origin, Event<T>} = 3,
		AxelarGateway: pallet_axelar_cgp::{Pallet, Call, Storage, Origin, Event<T>} = 4,
	}
);

pub const ALICE: AccountId = 1;

pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub fn event_exists<E: Into<RuntimeEvent>>(e: E) {
	let actual: Vec<RuntimeEvent> = frame_system::Pallet::<Runtime>::events()
		.iter()
		.map(|e| e.event.clone())
		.collect();

	let e: RuntimeEvent = e.into();
	let mut exists = false;
	for evt in actual {
		if evt == e {
			exists = true;
			break;
		}
	}
	assert!(exists);
}
