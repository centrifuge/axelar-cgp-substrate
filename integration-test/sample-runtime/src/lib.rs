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

use frame_support::{
	construct_runtime, parameter_types,
	traits::{Everything, Nothing},
	weights::{
		constants::{ExtrinsicBaseWeight, WEIGHT_PER_SECOND},
		Weight, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
	},
};
use frame_system::EnsureRoot;
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use smallvec::smallvec;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{Convert, IdentityLookup},
	AccountId32, Perbill,
};
use xcm::latest::prelude::*;
use xcm_builder::{
	AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, CurrencyAdapter, EnsureXcmOrigin, FixedWeightBounds,
	IsConcrete, LocationInverter, ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative,
	SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32,
	SovereignSignedViaLocation, TakeWeightCredit, UsingComponents,
};
use xcm_executor::{Config, XcmExecutor};

pub type AccountId = AccountId32;
pub type Balance = u128;
pub type Amount = i128;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type BaseCallFilter = Everything;
	type BlockHashCount = BlockHashCount;
	type BlockLength = ();
	type BlockNumber = u64;
	type BlockWeights = ();
	type DbWeight = ();
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type Header = Header;
	type Index = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SS58Prefix = ();
	type SystemWeightInfo = ();
	type Version = ();
}

parameter_types! {
	pub ExistentialDeposit: Balance = 1;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type AccountStore = System;
	type Balance = Balance;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

parameter_types! {
	pub const LocalChainId: u32 = 36;
}

impl axelar_cgp::Config for Runtime {
	type ApprovedCallForwarder = axelar_cgp::traits::RemoteCallForwarder<XcmRouter>;
	type ChainId = LocalChainId;
	type EnsureCallOrigin = axelar_cgp::traits::EnsureXcm;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type WeightInfo = ();
}

impl sample_receiver::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type WeightInfo = ();
}

impl parachain_info::Config for Runtime {}

parameter_types! {
	pub const RelayLocation: MultiLocation = MultiLocation::parent();
	pub const RelayNetwork: NetworkId = NetworkId::Kusama;
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}

pub type LocationToAccountId = (
	ParentIsPreset<AccountId>,
	SiblingParachainConvertsVia<Sibling, AccountId>,
	AccountId32Aliases<RelayNetwork, AccountId>,
);

pub type XcmOriginToCallOrigin = (
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
	XcmPassthrough<RuntimeOrigin>,
);

parameter_types! {
	pub const UnitWeightCost: u64 = 10;
	pub const MaxInstructions: u32 = 100;
}

/// Means for transacting assets on this chain.
pub type CurrencyTransactor = CurrencyAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<RelayLocation>,
	// Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports.
	(),
>;

pub type LocalAssetTransactor = CurrencyTransactor;

/// The means for routing XCM messages which are not for local execution into
/// the right message queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

pub type Barrier = (
	TakeWeightCredit,
	AllowTopLevelPaidExecutionFrom<Everything>,
	// Expected responses are OK.
	AllowKnownQueryResponses<PolkadotXcm>,
	// Subscriptions for version tracking are OK.
	AllowSubscriptionsFrom<Everything>,
);

pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
	type Balance = Balance;

	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// in Polkadot, extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
		// in Statemint, we map to 1/10 of that, or 1/100 CENT
		let p = polkadot_runtime_constants::currency::CENTS;
		let q = 100 * Balance::from(ExtrinsicBaseWeight::get().ref_time());
		smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

pub struct XcmConfig;
impl Config for XcmConfig {
	type AssetClaims = ();
	type AssetTransactor = LocalAssetTransactor;
	type AssetTrap = ();
	type Barrier = Barrier;
	type IsReserve = ();
	type IsTeleporter = ();
	type LocationInverter = LocationInverter<Ancestry>;
	type OriginConverter = XcmOriginToCallOrigin;
	type ResponseHandler = ();
	type RuntimeCall = RuntimeCall;
	type SubscriptionService = ();
	type Trader = UsingComponents<WeightToFee, RelayLocation, AccountId, Balances, ()>;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type XcmSender = XcmRouter;
}

parameter_types! {
	pub ReservedXcmpWeight: Weight = WEIGHT_PER_SECOND / 4;
	pub ReservedDmpWeight: Weight = WEIGHT_PER_SECOND / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type CheckAssociatedRelayNumber = cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
	type DmpMessageHandler = DmpQueue;
	type OnSystemEvent = ();
	type OutboundXcmpMessageSource = XcmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type RuntimeEvent = RuntimeEvent;
	type SelfParaId = ParachainInfo;
	type XcmpMessageHandler = XcmpQueue;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type ChannelInfo = ParachainSystem;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToCallOrigin;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type VersionWrapper = ();
	type WeightInfo = ();
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

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
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmReserveTransferFilter = Everything;
	type XcmRouter = XcmRouter;
	type XcmTeleportFilter = Nothing;

	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
}

pub struct AccountIdToMultiLocation;
impl Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
	fn convert(account: AccountId) -> MultiLocation {
		X1(Junction::AccountId32 {
			network: NetworkId::Any,
			id: account.into(),
		})
		.into()
	}
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Config, Event<T>},
		ParachainInfo: parachain_info::{Pallet, Storage, Config},
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>},
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>},
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin},
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin},
		AxelarGateway: axelar_cgp::{Pallet, Call, Storage, Origin, Event<T>},
		SampleReceiver: sample_receiver::{Pallet, Call, Origin, Event<T>},
	}
);
