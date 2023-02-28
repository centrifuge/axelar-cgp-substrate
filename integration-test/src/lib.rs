mod env;

#[cfg(test)]
mod tests {
    use super::*;
    use codec::Encode;
    use env::{Network, Para1, Para2};
    use frame_support::assert_ok;
    use xcm::latest::prelude::*;
    use xcm_emulator::TestExt;

    #[test]
    fn transact_xcm_origins() {
        Network::reset();

        Para1::execute_with(|| {
            let inner_call =
                sample_runtime::RuntimeCall::System(frame_system::Call::remark_with_event {
                    remark: vec![10],
                });

            assert_ok!(sample_runtime::PolkadotXcm::send_xcm(
                Here,
                MultiLocation::new(1, X1(Parachain(2))),
                Xcm(vec![Transact {
                    origin_type: OriginKind::SovereignAccount,
                    require_weight_at_most: 8_000_000_000,
                    call: inner_call.encode().into(),
                }]),
            ));
        });

        Para2::execute_with(|| {
            sample_runtime::System::events()
                .iter()
                .for_each(|r| println!(">>> {:#?}", r.event));

            assert!(sample_runtime::System::events().iter().any(|r| matches!(
                r.event,
                sample_runtime::RuntimeEvent::System(frame_system::Event::Remarked {
                    sender: _,
                    hash: _
                })
            )));
        });
    }
}
