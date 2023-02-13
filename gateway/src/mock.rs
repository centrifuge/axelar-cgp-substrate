use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU32, ConstU64, Everything},
    PalletId,
};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup};

use crate::traits::LocalCallForwarder;
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

parameter_types! {
    pub const GatewayPalletId: PalletId = crate::traits::GATEWAY_PALLET_ID;
    pub const ChainId: u16 = 36;
}

impl Config for Runtime {
    type PalletId = GatewayPalletId;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
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
        AxelarGateway: pallet_axelar_cgp::{Pallet, Call, Storage, Event<T>} = 3,
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
