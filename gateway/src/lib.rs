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
use frame_support::dispatch::RawOrigin;
use frame_support::traits::EnsureOrigin;
use frame_support::{transactional, PalletId};
pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod proof;

// ----------------------------------------------------------------------------
// Constants
// ----------------------------------------------------------------------------
pub const OLD_KEY_RETENTION: u64 = 16;

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
    use frame_support::dispatch::RawOrigin;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use pallet_utility::WeightInfo as UtilityWeightInfo;
    use sp_core::H256;
    use sp_runtime::traits::AccountIdConversion;
    use sp_runtime::ArithmeticError;

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
    pub trait Config: frame_system::Config + pallet_utility::Config {
        /// The Gateway Origin Pallet Identifier
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

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
    }

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

    // ------------------------------------------------------------------------
    // Pallet errors
    // ------------------------------------------------------------------------

    #[pallet::error]
    pub enum Error<T> {
        InvalidOperators,
        InvalidWeights,
        InvalidThreshold,
        DuplicateOperators,

        /// Failed to decode the proof
        FailedToDecodeProof,
        /// Proof validation failed
        InvalidProof,
    }

    // ------------------------------------------------------------------------
    // Pallet dispatchable functions
    // ------------------------------------------------------------------------

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // Weight definition taken from Substrate Utility.force_batch, not sure if there is a more succinct and maintainable
        // way to ensure the call is properly weighted
        #[pallet::weight({
            let dispatch_infos = calls.iter().map(|call| call.get_dispatch_info()).collect::<Vec<_>>();
            let dispatch_weight = dispatch_infos.iter()
                .map(|di| di.weight)
                .fold(Weight::zero(), |total: Weight, weight: Weight| total.saturating_add(weight))
                .saturating_add(<pallet_utility::weights::SubstrateWeight<T> as UtilityWeightInfo>::force_batch(calls.len() as u32));
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
            _proof: u64,
            calls: Vec<<T as pallet_utility::Config>::RuntimeCall>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_signed(origin)?;
            // PLACEHOLDER: Verify command batch proof
            pallet_utility::Pallet::<T>::force_batch(
                RawOrigin::Signed(Self::account_id()).into(),
                calls,
            )
        }

        #[pallet::weight(<T as pallet::Config>::WeightInfo::transfer_operatorship(new_operators.len() as u32))]
        #[transactional]
        pub fn transfer_operatorship(
            origin: OriginFor<T>,
            new_operators: Vec<[u8; 20]>,
            new_weights: Vec<u128>,
            new_threshold: u128,
        ) -> DispatchResult {
            // Ensure only gateway origin can call this
            let _ = EnsureGateway::<T>::ensure_origin(origin)?;

            let new_operator_hash =
                Self::validate_operatorship(new_operators, new_weights, new_threshold)?;

            ensure!(
                !<EpochForHash<T>>::contains_key(new_operator_hash),
                Error::<T>::DuplicateOperators
            );

            let epoch = <CurrentEpoch<T>>::get()
                .checked_add(1)
                .ok_or(ArithmeticError::Overflow)?;
            <CurrentEpoch<T>>::set(epoch);
            <HashForEpoch<T>>::set(epoch, new_operator_hash);
            <EpochForHash<T>>::set(new_operator_hash, epoch);

            Self::deposit_event(Event::OperatorshipTransferred {
                new_operator_hash,
                new_epoch: epoch,
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
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
    }
}
// end of 'pallet' module

pub struct EnsureGateway<T>(sp_std::marker::PhantomData<T>);
impl<T: pallet::Config> EnsureOrigin<T::RuntimeOrigin> for EnsureGateway<T> {
    type Success = T::AccountId;

    fn try_origin(o: T::RuntimeOrigin) -> Result<Self::Success, T::RuntimeOrigin> {
        let gateway_id = Pallet::<T>::account_id();
        o.into().and_then(|o| match o {
            RawOrigin::Signed(who) if who == gateway_id => Ok(gateway_id),
            r => Err(T::RuntimeOrigin::from(r)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn successful_origin() -> T::RuntimeOrigin {
        unimplemented!()
    }
}
