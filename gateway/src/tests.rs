use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use sp_core::H256;

#[test]
fn accounts_ordered() {
    ExtBuilder::default().build().execute_with(|| {
        // Empty Address error
        let mut input: Vec<[u8; 20]> = vec![[0; 20], [2; 20]];
        let result = AxelarGateway::is_sorted_asc_and_contains_no_duplicates(input);
        assert!(!result);

        // Wrong order - Desc
        input = vec![[2; 20], [1; 20]];
        let result = AxelarGateway::is_sorted_asc_and_contains_no_duplicates(input);
        assert!(!result);

        // Success
        input = vec![[1; 20], [2; 20]];
        let result = AxelarGateway::is_sorted_asc_and_contains_no_duplicates(input);
        assert!(result);
    });
}

#[test]
fn validate_operatorship_params() {
    ExtBuilder::default().build().execute_with(|| {
        // new_operators vector is empty
        assert_noop!(
            AxelarGateway::validate_operatorship(vec![], vec![], 0u128),
            Error::<Runtime>::InvalidOperators,
        );

        let mut new_operators: Vec<[u8; 20]> = vec![[1; 20]];
        let mut new_weights: Vec<u128> = vec![];
        // new_weights do not match new_operators length
        assert_noop!(
            AxelarGateway::validate_operatorship(new_operators, new_weights, 0u128),
            Error::<Runtime>::InvalidWeights,
        );

        new_operators = vec![[1; 20], [2; 20], [3; 20]];
        new_weights = vec![10u128, 10u128, 20u128];
        // new_weights will never reach threshold
        assert_noop!(
            AxelarGateway::validate_operatorship(
                new_operators.clone(),
                new_weights.clone(),
                50u128
            ),
            Error::<Runtime>::InvalidThreshold,
        );

        let result = AxelarGateway::validate_operatorship(new_operators, new_weights, 20u128);
        // success
        assert_ok!(result);
        // Hash precomputed for input from last values of new_operators+new_weights+threshold
        let precomputed_hash = H256::from([
            232, 1, 82, 130, 189, 175, 253, 64, 101, 205, 209, 35, 92, 250, 52, 60, 120, 107, 11,
            183, 201, 98, 82, 106, 176, 13, 108, 109, 18, 47, 214, 160,
        ]);
        assert_eq!(result.unwrap(), precomputed_hash)
    });
}

#[test]
fn transfer_operatorship() {
    ExtBuilder::default().build().execute_with(|| {
        // new_operators vector is empty
        assert_noop!(
            AxelarGateway::validate_operatorship(vec![], vec![], 0u128),
            Error::<Runtime>::InvalidOperators,
        );

        let new_operators = vec![[1; 20], [2; 20], [3; 20]];
        let new_weights = vec![10u128, 10u128, 20u128];
        let precomputed_hash = H256::from([
            232, 1, 82, 130, 189, 175, 253, 64, 101, 205, 209, 35, 92, 250, 52, 60, 120, 107, 11,
            183, 201, 98, 82, 106, 176, 13, 108, 109, 18, 47, 214, 160,
        ]);

        // Hash already exists
        let new_epoch = 1u64;
        EpochForHash::<Runtime>::insert(precomputed_hash, new_epoch);
        assert_noop!(
            AxelarGateway::transfer_operatorship(
                RuntimeOrigin::signed(ALICE),
                new_operators.clone(),
                new_weights.clone(),
                20u128
            ),
            Error::<Runtime>::DuplicateOperators,
        );

        // Remove hash
        EpochForHash::<Runtime>::remove(precomputed_hash);
        assert_ok!(AxelarGateway::transfer_operatorship(
            RuntimeOrigin::signed(ALICE),
            new_operators.clone(),
            new_weights.clone(),
            20u128
        ));

        //Check storages
        assert_eq!(CurrentEpoch::<Runtime>::get(), new_epoch);
        assert_eq!(HashForEpoch::<Runtime>::get(new_epoch), precomputed_hash);
        assert_eq!(EpochForHash::<Runtime>::get(precomputed_hash), new_epoch);

        event_exists(Event::<Runtime>::OperatorshipTransferred {
            new_operator_hash: precomputed_hash,
            new_epoch,
        });
    });
}
