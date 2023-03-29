use frame_support::traits::{Nothing, OriginTrait};
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU32, ConstU64, Everything},
};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup};
use std::marker::PhantomData;
use xcm::latest::prelude::*;
use xcm::opaque::latest::NetworkId;
use xcm_builder::{EnsureXcmOrigin, FixedWeightBounds, LocationInverter};

use crate::traits::{EnsureXcm, LocalCallForwarder};
use crate::{self as pallet_axelar_cgp, Config};

pub type AccountId = u64;
pub type BlockNumber = u64;

impl frame_system::Config for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type RuntimeCall = RuntimeCall;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type BlockWeights = ();
    type BlockLength = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = Everything;
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
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
    type RuntimeEvent = RuntimeEvent;
    type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmRouter = ();
    type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmExecuteFilter = Everything;
    type XcmExecutor = ();
    type XcmTeleportFilter = Nothing;
    type XcmReserveTransferFilter = Everything;
    type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
    type LocationInverter = LocationInverter<Ancestry>;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

// parameter_types! {
//     pub LocalParaId: u32 = ParachainInfo::parachain_id().into();
// }

impl Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type EnsureCallOrigin = EnsureXcm;
    type ChainId = ChainId;
    type ApprovedCallForwarder = LocalCallForwarder;
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
