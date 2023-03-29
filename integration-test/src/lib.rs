mod env;

#[cfg(test)]
mod tests {
    use super::*;
    use axelar_cgp::proof;
    use codec::Encode;
    use env::{Network, Para1, Para2};
    use ethabi::Token;
    use frame_support::{assert_noop, assert_ok};
    use sample_runtime::{AccountId, RuntimeOrigin};
    use sp_core::{keccak_256, H160, H256, U256};
    use sp_runtime::DispatchError::BadOrigin;
    use std::str::FromStr;
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

            let fee_asset = MultiAsset {
                id: Concrete(MultiLocation {
                    parents: 1,
                    interior: Junctions::Here,
                }),
                fun: Fungible(8_000_000_000),
            };

            assert_ok!(sample_runtime::PolkadotXcm::send_xcm(
                Here,
                MultiLocation::new(1, X1(Parachain(2))),
                Xcm(vec![
                    WithdrawAsset(fee_asset.clone().into()),
                    BuyExecution {
                        fees: fee_asset.into(),
                        weight_limit: WeightLimit::Unlimited,
                    },
                    Transact {
                        origin_type: OriginKind::SovereignAccount,
                        require_weight_at_most: 8_000_000_000,
                        call: inner_call.encode().into(),
                    }
                ]),
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

    #[test]
    fn xcm_call_contract() {
        Network::reset();

        Para1::execute_with(|| {
            let inner_call =
                sample_runtime::RuntimeCall::AxelarGateway(axelar_cgp::Call::call_contract {
                    destination_chain: String::from("ethereum"),
                    destination_contract_address: String::from(
                        "0x5f927395213ee6b95de97bddcb1b2b1c0f16844d",
                    ),
                    payload: vec![1; 32],
                });

            let fee_asset = MultiAsset {
                id: Concrete(MultiLocation {
                    parents: 1,
                    interior: Junctions::Here,
                }),
                fun: Fungible(8_000_000_000),
            };

            assert_ok!(sample_runtime::PolkadotXcm::send_xcm(
                Here,
                MultiLocation::new(1, X1(Junction::Parachain(2))),
                Xcm(vec![
                    WithdrawAsset(fee_asset.clone().into()),
                    BuyExecution {
                        fees: fee_asset.into(),
                        weight_limit: WeightLimit::Unlimited,
                    },
                    Transact {
                        origin_type: OriginKind::Xcm,
                        require_weight_at_most: 8_000_000_000,
                        call: inner_call.encode().into(),
                    }
                ]),
            ));
        });

        Para2::execute_with(|| {
            sample_runtime::System::events()
                .iter()
                .for_each(|r| println!(">>> {:#?}", r.event));

            assert!(sample_runtime::System::events().iter().any(|r| matches!(
                r.event,
                sample_runtime::RuntimeEvent::AxelarGateway(axelar_cgp::Event::ContractCall {
                    sender: _,
                    destination_chain: _,
                    destination_contract_address: _,
                    payload_hash: _,
                    payload: _
                })
            )));
        });
    }

    #[test]
    fn call_contract_wrong_origin() {
        Network::reset();
        let alice: AccountId = AccountId::from_str(
            "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
        )
        .unwrap();

        Para1::execute_with(|| {
            assert_noop!(
                sample_runtime::AxelarGateway::call_contract(
                    RuntimeOrigin::signed(alice),
                    String::from("ethereum"),
                    String::from("0x5f927395213ee6b95de97bddcb1b2b1c0f16844d"),
                    vec![1; 32]
                ),
                BadOrigin
            );
        });
    }

    #[test]
    fn forward_approved_call_xcm() {
        Network::reset();

        Para1::execute_with(|| {
            let alice: AccountId = AccountId::from_str(
                "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
            )
            .unwrap();

            let inner_call = sample_runtime::RuntimeCall::SampleReceiver(
                sample_receiver::Call::sample_final_call {
                    data: vec![1, 2, 3],
                },
            );
            let inner_call_bytes = inner_call.encode();

            let command_id = H256::random();
            let source_chain = String::from("ethereum");
            let source_address = String::from("0x5f927395213ee6b95de97bddcb1b2b1c0f16844d");
            let contract_address = H160::random();

            let call_hash = H256::from_slice(&ecdsa::to_eth_signed_message_hash(keccak_256(
                inner_call_bytes.as_slice(),
            )));
            let mut approved_call = command_id.encode();
            approved_call.append(&mut source_chain.encode());
            approved_call.append(&mut source_address.encode());
            approved_call.append(&mut contract_address.encode());
            approved_call.append(&mut call_hash.encode());

            let approved_call_hash = H256::from(keccak_256(approved_call.as_slice()));

            axelar_cgp::ContractCallApproved::<sample_runtime::Runtime>::set(
                approved_call_hash,
                (),
            );
            axelar_cgp::CommandExecuted::<sample_runtime::Runtime>::set(command_id, 2u32);

            assert_ok!(sample_runtime::AxelarGateway::forward_approved_call(
                RuntimeOrigin::signed(alice),
                command_id,
                source_chain,
                source_address,
                contract_address,
                inner_call_bytes
            ));
        });

        Para2::execute_with(|| {
            sample_runtime::System::events()
                .iter()
                .for_each(|r| println!(">>> {:#?}", r.event));

            assert!(sample_runtime::System::events().iter().any(|r| matches!(
                r.event,
                sample_runtime::RuntimeEvent::SampleReceiver(
                    sample_receiver::Event::SampleFinalCall {
                        sender: _,
                        data: _,
                        proxy_chain: _,
                    }
                )
            )));
        });
    }

    #[test]
    fn full_execute_flow() {
        Network::reset();

        let alice: AccountId = AccountId::from_str(
            "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
        )
        .unwrap();

        Para1::execute_with(|| {
            // This is the call to be executed in the final parachain
            let inner_call = sample_runtime::RuntimeCall::SampleReceiver(
                sample_receiver::Call::sample_final_call {
                    data: vec![1, 2, 3],
                },
            );
            let inner_call_bytes: Vec<u8> = inner_call.encode();

            /////////// Preparing Execute Batch Payload ///////////
            let source_chain = String::from("ethereum");
            let source_address = String::from("0x5f927395213ee6b95de97bddcb1b2b1c0f16844d");
            let contract_address = H160::random();
            let payload_hash = H256::from_slice(&ecdsa::to_eth_signed_message_hash(keccak_256(
                inner_call_bytes.as_slice(),
            )));
            let source_tx_hash = H256::random();
            let source_event_index = U256::from(100);
            let command_id = H256::random();

            // Preparing call to be executed in the Axelar gateway parachain
            let outer_call = sample_runtime::RuntimeCall::AxelarGateway(
                axelar_cgp::Call::approve_contract_call {
                    source_chain: source_chain.clone(),
                    source_address: source_address.clone(),
                    contract_address,
                    payload_hash,
                    source_tx_hash,
                    source_event_index,
                    command_id,
                },
            );

            // Preparing sign message, to be signed by the virtual operators
            let chain_id = 2_u32;
            let command_x: String = String::from("approveContractCall");
            let batch_msg: ethabi::Bytes = sample_runtime::AxelarGateway::abi_encode_batch_params(
                chain_id,
                vec![command_id.clone()],
                vec![command_x.clone()],
                vec![outer_call.clone()],
            );
            let sign_msg = ecdsa::to_eth_signed_message_hash(keccak_256(batch_msg.as_slice()));

            let operator_0 = ecdsa::generate_keypair();
            let operator_1 = ecdsa::generate_keypair();
            let operator_0_public =
                H160::from(H256::from_slice(keccak_256(&operator_0.0).as_slice()));
            let operator_1_public =
                H160::from(H256::from_slice(keccak_256(&operator_1.0).as_slice()));
            let sig_0 = ecdsa::sign_message(H256::from_slice(&sign_msg), &operator_0.1);
            let sig_1 = ecdsa::sign_message(H256::from_slice(&sign_msg), &operator_1.1);
            let proof_bytes = axelar_cgp::utils::proofs::encode(
                vec![
                    operator_0_public.to_fixed_bytes(),
                    operator_1_public.to_fixed_bytes(),
                ],
                vec![50u128, 50u128],
                50u128,
                vec![sig_0, sig_1],
            )
            .to_vec();

            let op0_addr = Token::Address(operator_0_public.to_fixed_bytes().into())
                .into_address()
                .unwrap();
            let op1_addr = Token::Address(operator_1_public.to_fixed_bytes().into())
                .into_address()
                .unwrap();
            let operators_hash =
                proof::operators_hash(vec![op0_addr, op1_addr], vec![50u128, 50u128], 50u128);

            // Initialize the Epoch for the above set of operators
            axelar_cgp::EpochForHash::<sample_runtime::Runtime>::insert(operators_hash, 100);
            axelar_cgp::CurrentEpoch::<sample_runtime::Runtime>::set(100);

            //////////////////// End Preparing batch Execute /////////////////////

            // Axelar Relayer triggers the batch execution
            assert_ok!(sample_runtime::AxelarGateway::execute(
                RuntimeOrigin::signed(alice.clone()),
                proof_bytes.clone(),
                chain_id,
                vec![command_id],
                vec![command_x.clone()],
                vec![outer_call.clone()]
            ));

            assert_eq!(
                axelar_cgp::CommandExecuted::<sample_runtime::Runtime>::get(command_id),
                chain_id
            );
            assert!(sample_runtime::System::events().iter().any(|r| matches!(
                r.event,
                sample_runtime::RuntimeEvent::AxelarGateway(
                    axelar_cgp::Event::ContractCallApproved { .. }
                )
            )));

            // Now that call has been approved, Axelar Relayer forwards the final execution
            assert_ok!(sample_runtime::AxelarGateway::forward_approved_call(
                RuntimeOrigin::signed(alice),
                command_id,
                source_chain.clone(),
                source_address.clone(),
                contract_address,
                inner_call_bytes
            ));

            let mut call_hash_payload = command_id.encode();
            call_hash_payload.append(&mut source_chain.encode());
            call_hash_payload.append(&mut source_address.encode());
            call_hash_payload.append(&mut contract_address.encode());
            call_hash_payload.append(&mut payload_hash.encode());

            let approved_call_hash = H256::from(keccak_256(call_hash_payload.as_slice()));

            // After Contract Call approved, the call hash has been removed, so there is no collisions with retrials from the Axelar network
            assert!(
                !axelar_cgp::ContractCallApproved::<sample_runtime::Runtime>::contains_key(
                    approved_call_hash
                )
            );
        });

        Para2::execute_with(|| {
            sample_runtime::System::events()
                .iter()
                .for_each(|r| println!(">>> {:#?}", r.event));

            // Final call successfully executed across the "wire"
            assert!(sample_runtime::System::events().iter().any(|r| matches!(
                r.event,
                sample_runtime::RuntimeEvent::SampleReceiver(
                    sample_receiver::Event::SampleFinalCall { .. }
                )
            )));
        });
    }
}
