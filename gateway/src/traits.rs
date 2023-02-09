//! Traits used by the Axelar pallet

// ----------------------------------------------------------------------------
// Module imports and re-exports
// ----------------------------------------------------------------------------

use frame_support::PalletId;
use std::marker::PhantomData;
// Frame, system and frame primitives
use crate::Config;
use crate::Error::ErrorForwarding;
use codec::Decode;
use frame_support::dispatch::{DispatchResult, RawOrigin};
use frame_support::weights::Weight;
use sp_core::H160;
use sp_runtime::{traits::Dispatchable, DispatchError};
use xcm::latest::prelude::*;
use xcm::latest::Xcm;
use xcm::prelude::DescendOrigin;

// ----------------------------------------------------------------------------
// Traits declaration
// ----------------------------------------------------------------------------

pub const GATEWAY_PALLET_ID: PalletId = PalletId(*b"axgteway");

/// Weight information for pallet extrinsics
///
/// Weights are calculated using runtime benchmarking features.
/// See [`benchmarking`] module for more information.
pub trait WeightInfo {
    fn execute(c: u32) -> Weight;
    fn transfer_operatorship(c: u32) -> Weight;
    fn approve_contract_call() -> Weight;
    fn forward_approved_call() -> Weight;
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
}

pub trait ApprovedCallForwarder<AccountId, T> {
    fn is_local() -> bool;
    fn do_forward(
        who: AccountId,
        source_chain: String,
        source_address: String,
        contract_address: H160,
        dest: MultiLocation,
        call: Vec<u8>,
    ) -> DispatchResult;
}

/// Local Forwarder Default Implementation
pub struct ApprovedCallLocalForwarder;
impl<T: Config> ApprovedCallForwarder<T::AccountId, T> for ApprovedCallLocalForwarder {
    fn is_local() -> bool {
        true
    }

    fn do_forward(
        who: T::AccountId,
        _source_chain: String,
        _source_address: String,
        _contract_address: H160,
        _dest: MultiLocation,
        call: Vec<u8>,
    ) -> DispatchResult {
        match <T as Config>::RuntimeCall::decode(&mut &call[..]) {
            Ok(final_call) => {
                final_call
                    .dispatch(RawOrigin::Signed(who).into())
                    .map(|_| ())
                    .map_err(|e| e.error)?;
                Ok(())
            }
            Err(_) => Err(DispatchError::CannotLookup),
        }
    }
}

/// XCM Forwarder Implementation
pub struct ApprovedCallRemoteForwarder<XcmSender>(PhantomData<XcmSender>);
impl<T: Config, XcmSender: SendXcm> ApprovedCallForwarder<T::AccountId, T>
    for ApprovedCallRemoteForwarder<XcmSender>
{
    fn is_local() -> bool {
        false
    }

    fn do_forward(
        _who: T::AccountId,
        source_chain: String,
        source_address: String,
        _contract_address: H160,
        dest: MultiLocation,
        call: Vec<u8>,
    ) -> DispatchResult {
        // let function_prefx = [0,1]; // configurable per sovereign chain
        // let call_arguments = _source_chain.append(_source_address.encode().append(_call));

        // TODO: xcm v2 - review
        let eth_junction = Junction::AccountKey20 {
            network: NetworkId::Named(
                source_chain
                    .into_bytes()
                    .try_into()
                    .expect("shorter than length limit; qed"),
            ),
            key: H160::from_slice(&hex::decode(source_address).expect("")).to_fixed_bytes(),
        };

        // TODO: dummy multilocation - review
        let fee_asset = MultiAsset {
            id: Concrete(MultiLocation {
                parents: 0,
                interior: Junctions::Here,
            }),
            fun: Fungible(8_000_000_000),
        };

        // WIP idea
        let transact_message = Xcm(vec![
            // use xcm v3 when ready
            DescendOrigin(X1(eth_junction)),
            WithdrawAsset(fee_asset.clone().into()),
            // TODO: dummy weight - review
            BuyExecution {
                fees: fee_asset.into(),
                weight_limit: WeightLimit::Unlimited,
            },
            // TODO: dummy transact params - review
            Transact {
                origin_type: OriginKind::SovereignAccount,
                require_weight_at_most: 8_000_000_000,
                call: call.into(),
            },
        ]);

        XcmSender::send_xcm(dest, transact_message).map_err(|_| ErrorForwarding::<T>)?;

        Ok(())
    }
}
