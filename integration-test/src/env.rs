use frame_support::{pallet_prelude::Weight, traits::GenesisBuild};
use sp_runtime::AccountId32;

use xcm_emulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};

pub const PARA_1_ID: u32 = 1;
pub const PARA_2_ID: u32 = 2;

decl_test_relay_chain! {
    pub struct KusamaNet {
        Runtime = kusama_runtime::Runtime,
        XcmConfig = kusama_runtime::xcm_config::XcmConfig,
        new_ext = kusama_ext(),
    }
}

decl_test_parachain! {
    pub struct Para1 {
        Runtime = sample_runtime::Runtime,
        RuntimeOrigin = sample_runtime::RuntimeOrigin,
        XcmpMessageHandler = sample_runtime::XcmpQueue,
        DmpMessageHandler = sample_runtime::DmpQueue,
        new_ext = sample_runtime_ext(PARA_1_ID),
    }
}

decl_test_parachain! {
    pub struct Para2 {
        Runtime = sample_runtime::Runtime,
        RuntimeOrigin = sample_runtime::RuntimeOrigin,
        XcmpMessageHandler = sample_runtime::XcmpQueue,
        DmpMessageHandler = sample_runtime::DmpQueue,
        new_ext = sample_runtime_ext(PARA_2_ID),
    }
}

decl_test_network! {
    pub struct Network {
        relay_chain = KusamaNet,
        parachains = vec![
            (1, Para1),
            (2, Para2),
        ],
    }
}

pub const ALICE: AccountId32 = AccountId32::new([0u8; 32]);
pub const INITIAL_BALANCE: u128 = 1_000_000_000_000;

pub fn sample_runtime_ext(para_id: u32) -> sp_io::TestExternalities {
    use sample_runtime::{Runtime, System};

    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap();

    let parachain_info_config = parachain_info::GenesisConfig {
        parachain_id: para_id.into(),
    };

    <parachain_info::GenesisConfig as GenesisBuild<Runtime, _>>::assimilate_storage(
        &parachain_info_config,
        &mut t,
    )
    .unwrap();

    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![(ALICE, INITIAL_BALANCE)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn default_parachains_host_configuration(
) -> polkadot_runtime_parachains::configuration::HostConfiguration<
    polkadot_primitives::v2::BlockNumber,
> {
    use polkadot_primitives::v2::{MAX_CODE_SIZE, MAX_POV_SIZE};

    polkadot_runtime_parachains::configuration::HostConfiguration {
        minimum_validation_upgrade_delay: 5,
        validation_upgrade_cooldown: 10u32,
        validation_upgrade_delay: 10,
        code_retention_period: 1200,
        max_code_size: MAX_CODE_SIZE,
        max_pov_size: MAX_POV_SIZE,
        max_head_data_size: 32 * 1024,
        group_rotation_frequency: 20,
        chain_availability_period: 4,
        thread_availability_period: 4,
        max_upward_queue_count: 8,
        max_upward_queue_size: 1024 * 1024,
        max_downward_message_size: 1024,
        ump_service_total_weight: Weight::from_ref_time(4 * 1_000_000_000),
        max_upward_message_size: 50 * 1024,
        max_upward_message_num_per_candidate: 5,
        hrmp_sender_deposit: 0,
        hrmp_recipient_deposit: 0,
        hrmp_channel_max_capacity: 8,
        hrmp_channel_max_total_size: 8 * 1024,
        hrmp_max_parachain_inbound_channels: 4,
        hrmp_max_parathread_inbound_channels: 4,
        hrmp_channel_max_message_size: 1024 * 1024,
        hrmp_max_parachain_outbound_channels: 4,
        hrmp_max_parathread_outbound_channels: 4,
        hrmp_max_message_num_per_candidate: 5,
        dispute_period: 6,
        no_show_slots: 2,
        n_delay_tranches: 25,
        needed_approvals: 2,
        relay_vrf_modulo_samples: 2,
        zeroth_delay_tranche_width: 0,
        ..Default::default()
    }
}

pub fn kusama_ext() -> sp_io::TestExternalities {
    use kusama_runtime::{Runtime, System};

    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap();

    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![(ALICE, INITIAL_BALANCE)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    polkadot_runtime_parachains::configuration::GenesisConfig::<Runtime> {
        config: default_parachains_host_configuration(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
