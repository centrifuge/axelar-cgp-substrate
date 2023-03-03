//! Traits used by the Axelar pallet

// ----------------------------------------------------------------------------
// Module imports and re-exports
// ----------------------------------------------------------------------------

use std::marker::PhantomData;
// Frame, system and frame primitives
use crate::Error::ErrorForwarding;
use crate::{pallet, Config, MultiAddress, RawOrigin};
use codec::Decode;
use frame_support::dispatch::DispatchResult;
use frame_support::weights::Weight;
use sp_core::bounded::WeakBoundedVec;
use sp_core::{ConstU32, H160};
use sp_runtime::{traits::Dispatchable, DispatchError};
use std::str::FromStr;
use xcm::latest::prelude::*;
use xcm::latest::Xcm;
use xcm::prelude::DescendOrigin;

use crate::utils::vec_to_fixed_array;

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
        contract_address: H160,
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
        _contract_address: H160,
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
        _contract_address: H160,
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
            // use xcm v3 when ready
            // UniversalOrigin(GlobalConsensus(NetworkId::Ethereum(id)))
            DescendOrigin(X1(origin_junction)),
            WithdrawAsset(fee_asset.clone().into()),
            // TODO: review unlimited weight
            BuyExecution {
                fees: fee_asset.into(),
                weight_limit: WeightLimit::Unlimited,
            },
            RefundSurplus,
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
