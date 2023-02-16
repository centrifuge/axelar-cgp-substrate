//! # Axelar CGP
//!
//! This pallet implements the Axelar Cross-Chain Gateway Protocol

// Ensure we're `no_std` when compiling for WebAssembly.
#![cfg_attr(not(feature = "std"), no_std)]

// ----------------------------------------------------------------------------
// Module imports and re-exports
// ----------------------------------------------------------------------------

// Pallet traits declaration
pub mod traits;

// Re-export pallet components in crate namespace (for runtime construction)
use crate::traits::WeightInfo;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::{extract_actual_weight, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::EnsureOrigin;
pub use pallet::*;
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod proof;

// ----------------------------------------------------------------------------
// Constants
// ----------------------------------------------------------------------------
pub const OLD_KEY_RETENTION: u64 = 16;

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum RawOrigin {
    Bridge,
}

// ----------------------------------------------------------------------------
// Pallet module
// ----------------------------------------------------------------------------

// Axelar pallet module
//
// The name of the pallet is provided by `construct_runtime` and is used as
// the unique identifier for the pallet's storage. It is not defined in the
// pallet itself.
#[frame_support::pallet]
pub mod pallet {
    use crate::proof::operators_hash;
    use ethabi::Token;
    use frame_support::pallet_prelude::*;
    use frame_support::traits::IsSubType;
    use frame_system::pallet_prelude::*;
    use sp_core::{keccak_256, H160, H256, U256};
    use sp_runtime::traits::Dispatchable;
    use sp_runtime::ArithmeticError;
    use traits::CallForwarder;

    use super::*;

    // Axelar pallet type declaration.
    //
    // This structure is a placeholder for traits and functions implementation
    // for the pallet.
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // ------------------------------------------------------------------------
    // Pallet configuration
    // ------------------------------------------------------------------------

    /// Axelar pallet's configuration trait.
    ///
    /// Associated types and constants are declared in this trait. If the pallet
    /// depends on other super-traits, the latter must be added to this trait,
    /// Note that [`frame_system::Config`] must always be included.
    #[pallet::config]
    pub trait Config:
    // Assert our origins are the same so we can bound them. This is a workaround for associated type bounds being unstable - See RFC 2289.
    // Ideally this would be:
    //    frame_system::Config<RuntimeOrigin: From<RawOrigin> + Into<Result<RawOrigin, <Self as frame_system::Config>::RuntimeOrigin>>>
    // Then we wouldn't need our own RuntimeOrigin type at all to handle adding bounds.
    frame_system::Config<RuntimeOrigin = <Self as Config>::RuntimeOrigin>
    {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The overarching origin type.
        // The `frame_system::RawOrigin` related bounds are needed because the compiler isn't smart enough to infer
        // that the `frame_system::Config::RawOrigin` are implied by our type-equality above. More may be needed to
        // make everything work, but the one here gets `ensure_signed` to behave.
        type RuntimeOrigin: From<RawOrigin>
            + Into<Result<RawOrigin, <Self as Config>::RuntimeOrigin>>
            + Into<
                Result<
                    frame_system::RawOrigin<<Self as frame_system::Config>::AccountId>,
                    <Self as Config>::RuntimeOrigin,
                >,
            >;

        /// The overarching call type.
        type RuntimeCall: Parameter
            + Dispatchable<RuntimeOrigin = <Self as Config>::RuntimeOrigin, PostInfo = PostDispatchInfo>
            + GetDispatchInfo
            + From<frame_system::Call<Self>>
            + IsSubType<Call<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeCall>;

        /// Self chain Identifier
        #[pallet::constant]
        type ChainId: Get<u32>;

        /// The forwarder for approved calls
        type ApprovedCallForwarder: CallForwarder<Self>;

        /// Weight information for extrinsics in this pallet
        type WeightInfo: WeightInfo;
    }

    // ------------------------------------------------------------------------
    // Pallet events
    // ------------------------------------------------------------------------

    // The macro generates event metadata and derive Clone, Debug, Eq, PartialEq and Codec
    #[pallet::event]
    // The macro generates a function on Pallet to deposit an event
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        OperatorshipTransferred {
            new_operator_hash: H256,
            new_epoch: u64,
        },
        ContractCallApproved {
            command_id: H256,
            source_chain: String,
            source_address: String,
            contract_address: H160,
            payload_hash: H256,
            source_tx_hash: H256,
            source_event_index: U256,
        },
        BatchCompleted,
        BatchCompletedWithErrors,
        ItemCompleted,
        ItemFailed {
            index: u32,
            error: DispatchError,
        },
    }

    #[pallet::origin]
    pub type Origin = RawOrigin;

    // Code block taken from: https://github.com/paritytech/substrate/blob/ee316317b85b2f65fc022b27bbfefcd42b6560ae/frame/utility/src/lib.rs#L128
    // Align the call size to 1KB. As we are currently compiling the runtime for native/wasm
    // the `size_of` of the `Call` can be different. To ensure that this don't leads to
    // mismatches between native/wasm or to different metadata for the same runtime, we
    // align the call size. The value is chosen big enough to hopefully never reach it.
    const CALL_ALIGN: u32 = 1024;

    // ------------------------------------------------------------------------
    // Pallet storage
    // ------------------------------------------------------------------------

    #[pallet::storage]
    #[pallet::getter(fn current_epoch)]
    pub(super) type CurrentEpoch<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn hash_for_epoch)]
    pub(super) type HashForEpoch<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        // Epoch
        u64,
        // Operators Hash
        H256,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn epoch_for_hash)]
    pub(super) type EpochForHash<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        // Operators Hash
        H256,
        // Epoch
        u64,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn command_executed)]
    pub(super) type CommandExecuted<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        // Command Id
        H256,
        // Destination Parachain Id
        u32,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn contract_call_approved)]
    pub(super) type ContractCallApproved<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        // Hash of contract call uniqueness
        H256,
        // Empty
        (),
        ValueQuery,
    >;

    // ------------------------------------------------------------------------
    // Pallet errors
    // ------------------------------------------------------------------------

    #[pallet::error]
    pub enum Error<T> {
        InvalidOperators,
        InvalidWeights,
        InvalidThreshold,
        DuplicateOperators,
        FailedToDecodeProof,
        InvalidProof,
        NotActiveOperators,
        TooManyCalls,
        CommandIdsLengthMismatch,
        WrongChainId,
        ContractCallNotApproved,
        ErrorForwarding,
    }

    // ------------------------------------------------------------------------
    // Pallet dispatchable functions
    // ------------------------------------------------------------------------

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Executes a batch of calls previously approved by the Axelar Consensus
        ///
        /// command_ids: ordered list of uuid that identify each command within the batch
        /// commands: ordered list of which command triggered the call. It is a fixed set of options burnToken|contractCall|...
        /// calls: the actual call to be executed in the gateway contract (it contains as well any other final calls)
        ///
        /// The weight definition taken from Substrate Utility.force_batch, not sure if there is a more succinct and maintainable
        /// way to ensure the call is properly weighted
        #[pallet::weight({
            let dispatch_infos = calls.iter().map(|call| call.get_dispatch_info()).collect::<Vec<_>>();
            let dispatch_weight = dispatch_infos.iter()
                .map(|di| di.weight)
                .fold(Weight::zero(), |total: Weight, weight: Weight| total.saturating_add(weight))
                .saturating_add(<T as pallet::Config>::WeightInfo::execute(calls.len() as u32));
            let dispatch_class = {
                let all_operational = dispatch_infos.iter()
                    .map(|di| di.class)
                    .all(|class| class == DispatchClass::Operational);
                if all_operational {
                    DispatchClass::Operational
                } else {
                    DispatchClass::Normal
                }
            };
            (dispatch_weight, dispatch_class)
        })]
        pub fn execute(
            origin: OriginFor<T>,
            proof: Vec<u8>,
            chain_id: u32,
            command_ids: Vec<H256>,
            commands: Vec<String>,
            calls: Vec<<T as Config>::RuntimeCall>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_signed(origin)?;

            // TODO: Once XCM is enabled this check might not make sense
            ensure!(chain_id == T::ChainId::get(), Error::<T>::WrongChainId);

            ensure!(
                calls.len() == command_ids.len(),
                Error::<T>::CommandIdsLengthMismatch
            );

            let payload = Self::abi_encode_batch_params(
                chain_id.clone(),
                command_ids.clone(),
                commands.clone(),
                calls.clone(),
            );
            // TODO: Double check on Axelar if they always prepend the eth prefix
            let payload_hash = H256::from_slice(&ecdsa::to_eth_signed_message_hash(keccak_256(
                payload.as_slice(),
            )));
            let mut is_active_operators = Self::validate_proof(payload_hash, &proof)?;

            // Code simplified taken from https://github.com/paritytech/substrate/blob/ee316317b85b2f65fc022b27bbfefcd42b6560ae/frame/utility/src/lib.rs#L440
            let calls_len = calls.len();
            ensure!(
                calls_len <= Self::batched_calls_limit() as usize,
                Error::<T>::TooManyCalls
            );

            // Track the actual weight of each of the batch calls.
            let mut weight = Weight::zero();
            // Track failed dispatch occur.
            let mut has_error = false;
            for (idx, call) in calls.into_iter().enumerate() {
                if CommandExecuted::<T>::contains_key(command_ids[idx]) {
                    continue;
                }

                match call.is_sub_type() {
                    Some(Call::transfer_operatorship { .. }) => {
                        if !is_active_operators {
                            continue;
                        }
                        is_active_operators = false;
                    }
                    Some(Call::approve_contract_call { .. }) => {}
                    _ => continue,
                }

                let info = call.get_dispatch_info();
                CommandExecuted::<T>::set(command_ids[idx], chain_id);

                let result = call.dispatch(RawOrigin::Bridge.into());
                // Add the weight of this call.
                weight = weight.saturating_add(extract_actual_weight(&result, &info));
                if let Err(e) = result {
                    has_error = true;
                    CommandExecuted::<T>::remove(command_ids[idx]);
                    Self::deposit_event(Event::ItemFailed {
                        index: idx as u32,
                        error: e.error,
                    });
                } else {
                    Self::deposit_event(Event::ItemCompleted);
                }
            }

            if has_error {
                Self::deposit_event(Event::BatchCompletedWithErrors);
            } else {
                Self::deposit_event(Event::BatchCompleted);
            }

            let base_weight = <T as pallet::Config>::WeightInfo::execute(calls_len as u32);
            Ok(Some(base_weight.saturating_add(weight)).into())
        }

        #[pallet::weight(<T as pallet::Config>::WeightInfo::transfer_operatorship(new_operators.len() as u32))]
        pub fn transfer_operatorship(
            origin: OriginFor<T>,
            new_operators: Vec<[u8; 20]>,
            new_weights: Vec<u128>,
            new_threshold: u128,
        ) -> DispatchResult {
            // Ensure only gateway origin can call this
            let _ = EnsureGateway::ensure_origin(origin)?;

            let new_operator_hash =
                Self::validate_operatorship(new_operators, new_weights, new_threshold)?;

            ensure!(
                !EpochForHash::<T>::contains_key(new_operator_hash),
                Error::<T>::DuplicateOperators
            );

            let epoch = CurrentEpoch::<T>::get()
                .checked_add(1)
                .ok_or(ArithmeticError::Overflow)?;
            CurrentEpoch::<T>::set(epoch);
            HashForEpoch::<T>::set(epoch, new_operator_hash);
            EpochForHash::<T>::set(new_operator_hash, epoch);

            Self::deposit_event(Event::OperatorshipTransferred {
                new_operator_hash,
                new_epoch: epoch,
            });

            Ok(())
        }

        #[pallet::weight(<T as pallet::Config>::WeightInfo::approve_contract_call())]
        pub fn approve_contract_call(
            origin: OriginFor<T>,
            source_chain: String,
            source_address: String,
            contract_address: H160,
            payload_hash: H256,
            source_tx_hash: H256,
            source_event_index: U256,
            command_id: H256,
        ) -> DispatchResult {
            // Ensure only gateway origin can call this
            let _ = EnsureGateway::ensure_origin(origin)?;

            let mut payload = command_id.encode();
            payload.append(&mut source_chain.encode());
            payload.append(&mut source_address.encode());
            payload.append(&mut contract_address.encode());
            payload.append(&mut payload_hash.encode());

            ContractCallApproved::<T>::set(H256::from(keccak_256(payload.as_slice())), ());

            Self::deposit_event(Event::ContractCallApproved {
                command_id,
                source_chain,
                source_address,
                contract_address,
                payload_hash,
                source_tx_hash,
                source_event_index,
            });

            Ok(())
        }

        #[pallet::weight({
            let total_weight = <T as pallet::Config>::WeightInfo::forward_approved_call();
            if <T as pallet::Config>::ApprovedCallForwarder::is_local() {
                let cc = <T as Config>::RuntimeCall::decode(&mut &call[..]).unwrap();
                total_weight.saturating_add(cc.get_dispatch_info().weight);
            }
            total_weight
        })]
        pub fn forward_approved_call(
            origin: OriginFor<T>,
            command_id: H256,
            source_chain: String,
            source_address: String,
            contract_address: H160,
            call: Vec<u8>,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;

            // TODO: keccak_256 is the Axelar Gateway standard hashing at origin on EVM chains, check if it is consistent in every contractCall on every chain
            let call_hash = H256::from_slice(&ecdsa::to_eth_signed_message_hash(keccak_256(
                call.as_slice(),
            )));
            // TODO: Does it need to be ABI encoded?
            let mut approved_call = command_id.encode();
            approved_call.append(&mut source_chain.encode());
            approved_call.append(&mut source_address.encode());
            approved_call.append(&mut contract_address.encode());
            approved_call.append(&mut call_hash.encode());

            let approved_call_hash = H256::from(keccak_256(approved_call.as_slice()));

            // Ensure the call has been approved by the bridge beforehand
            ensure!(
                ContractCallApproved::<T>::contains_key(approved_call_hash),
                Error::<T>::ContractCallNotApproved
            );

            ContractCallApproved::<T>::remove(approved_call_hash);

            // Fetch Parachain Id previously stored when executing command
            let dest = CommandExecuted::<T>::get(command_id);

            T::ApprovedCallForwarder::do_forward(
                RawOrigin::Bridge.into(),
                source_chain,
                source_address,
                contract_address,
                dest,
                call,
            )?;

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        // Code taken from https://github.com/paritytech/substrate/blob/ee316317b85b2f65fc022b27bbfefcd42b6560ae/frame/utility/src/lib.rs#L133
        fn batched_calls_limit() -> u32 {
            let allocator_limit = sp_core::MAX_POSSIBLE_ALLOCATION;
            let call_size =
                ((sp_std::mem::size_of::<<T as Config>::RuntimeCall>() as u32 + CALL_ALIGN - 1)
                    / CALL_ALIGN)
                    * CALL_ALIGN;
            // The margin to take into account vec doubling capacity.
            let margin_factor = 3;

            allocator_limit / margin_factor / call_size
        }

        pub fn validate_operatorship(
            new_operators: Vec<[u8; 20]>,
            new_weights: Vec<u128>,
            new_threshold: u128,
        ) -> Result<H256, DispatchError> {
            let operators_length = new_operators.len();
            let weights_length = new_weights.len();

            ensure!(
                operators_length != 0
                    && Self::is_sorted_asc_and_contains_no_duplicates(new_operators.clone()),
                Error::<T>::InvalidOperators
            );
            ensure!(
                operators_length == weights_length,
                Error::<T>::InvalidWeights
            );

            let mut total_weight = 0;
            for i in 0..weights_length {
                total_weight += new_weights[i];
            }

            ensure!(
                new_threshold != 0 && total_weight >= new_threshold,
                Error::<T>::InvalidThreshold
            );

            let mut operators_token: Vec<Token> = vec![];
            for i in 0..operators_length {
                operators_token.push(Token::Address(new_operators[i].into()));
            }

            let mut weights_token: Vec<Token> = vec![];
            for i in 0..weights_length {
                weights_token.push(Token::Uint(new_weights[i].into()));
            }

            let params = ethabi::encode(&[
                Token::Array(operators_token),
                Token::Array(weights_token),
                Token::Uint(new_threshold.into()),
            ]);

            Ok(sp_io::hashing::keccak_256(&params).into())
        }

        pub fn is_sorted_asc_and_contains_no_duplicates(accounts: Vec<[u8; 20]>) -> bool {
            for i in 0..accounts.len() - 1 {
                if accounts[i] >= accounts[i + 1] {
                    return false;
                }
            }

            accounts[0] != [0; 20]
        }

        pub fn validate_proof(msg_hash: H256, raw_proof: &[u8]) -> Result<bool, DispatchError> {
            let proof = proof::decode(raw_proof).map_err(|_| Error::<T>::FailedToDecodeProof)?;
            let operators_hash = operators_hash(
                proof.operators.clone(),
                proof.weights.clone(),
                proof.threshold,
            );
            let operators_epoch = <EpochForHash<T>>::get(operators_hash);
            let current_epoch = <CurrentEpoch<T>>::get();

            ensure!(
                Self::valid_operators(operators_epoch, current_epoch),
                Error::<T>::InvalidOperators
            );

            proof::validate_signatures(msg_hash, proof).map_err(|_| Error::<T>::InvalidProof)?;

            Ok(operators_epoch == current_epoch)
        }

        /// Check if the operators are allowed to execute.
        /// Execution is allowed if
        ///   - The `operators_epoch` is not 0
        ///   - The `operators_epoch` is not expired, i.e., it's within the OLD_KEY_RETENTION period.
        fn valid_operators(operators_epoch: u64, current_epoch: u64) -> bool {
            operators_epoch != 0 && current_epoch - operators_epoch < OLD_KEY_RETENTION
        }

        /// Encodes batch params to ABI, so proof can be verified
        pub fn abi_encode_batch_params(
            chain_id: u32,
            command_ids: Vec<H256>,
            commands: Vec<String>,
            calls: Vec<<T as Config>::RuntimeCall>,
        ) -> ethabi::Bytes {
            // TODO: verify calling encode() on H256 doesnt double encode
            let command_ids_token: Vec<Token> = command_ids
                .into_iter()
                .map(|x| Token::FixedBytes(x.encode()))
                .collect();
            let commands_token: Vec<Token> = commands
                .into_iter()
                .map(|x| Token::String(x.into()))
                .collect();
            let calls_token: Vec<Token> = calls
                .into_iter()
                .map(|x| Token::Bytes(x.encode().into()))
                .collect();

            ethabi::encode(&[
                Token::Uint(ethabi::Uint::from(chain_id)),
                Token::Array(command_ids_token),
                Token::Array(commands_token),
                Token::Array(calls_token),
            ])
        }
    }
}
// end of 'pallet' module

pub struct EnsureGateway;
impl<O: Into<Result<RawOrigin, O>> + From<RawOrigin>> EnsureOrigin<O> for EnsureGateway {
    type Success = ();
    fn try_origin(o: O) -> Result<Self::Success, O> {
        o.into().and_then(|o| match o {
            RawOrigin::Bridge => Ok(()),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn successful_origin() -> Result<O, ()> {
        unimplemented!()
    }
}
