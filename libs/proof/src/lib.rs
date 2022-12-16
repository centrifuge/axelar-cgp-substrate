//! # Axelar Cross-Chain Gateway Protocol
//!
//! This pallet implements the proof validation logic of the Axelar CGP.
//!
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::boxed::Box;
use ethabi::ParamType;

use sp_core::H256;
use sp_std::vec::Vec;

// ----------------------------------------------------------------------------
// Custom types and type aliases
// ----------------------------------------------------------------------------
type Bytes = [u8];
type Bytes32 = [u8; 32];
type EthAddress = [u8; 20];

pub struct OperatorsState {
    pub hash: H256,
    pub epoch: u64,
}

/// @dev This function takes messageHash and proof data and reverts if proof is invalid
/// @return True if provided operators are the current ones
/// Original implementation in Solidity:
/// https://github.com/axelarnetwork/axelar-cgp-solidity/blob/main/contracts/auth/AxelarAuthWeighted.sol#L28
pub fn validate(_msg_hash: H256, proof: &Bytes, state: OperatorsState) -> bool {
    // (address[] memory operators, uint256[] memory weights, uint256 threshold, bytes[] memory signatures) = abi.decode(
    //     proof,
    //     (address[], uint256[], uint256, bytes[])
    // );

    // let (operators, weights, threshold, signatures)
    let xs = decode(proof).expect("todo(nuno): return error instead");

    // bytes32 operatorsHash = keccak256(abi.encode(operators, weights, threshold));
    // uint256 operatorsEpoch = epochForHash[operatorsHash];
    // uint256 epoch = currentEpoch;

    //
    // if (operatorsEpoch == 0 || epoch - operatorsEpoch >= OLD_KEY_RETENTION) revert InvalidOperators();
    //
    // _validateSignatures(messageHash, operators, weights, threshold, signatures);
    //
    // return operatorsEpoch == epoch;

    false
}

fn decode(payload: &[u8]) -> Result<Vec<ethabi::Token>, ethabi::Error> {
    ethabi::decode(
        &[
            // operator's addresses
            ParamType::Array(Box::new(ParamType::Address)),
            // weights
            ParamType::Array(Box::new(ParamType::Uint(usize::MAX))),
            // threshold
            ParamType::Uint(usize::MAX),
            // signatures
            ParamType::Array(Box::new(ParamType::Bytes)),
        ],
        payload,
    )
}

// ----------------------------------------------------------------------------
// Tests
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use ethabi::Token;
    use sp_std::vec;

    fn encode(
        operators: Vec<[u8; 20]>,
        weights: Vec<u128>,
        threshold: u128,
        signatures: Vec<Vec<u8>>,
    ) -> ethabi::Bytes {
        let operators_token = operators
            .into_iter()
            .map(|x| Token::Address(x.into()))
            .collect();
        let weights_token = weights.into_iter().map(|x| Token::Uint(x.into())).collect();
        let signatures_token = signatures.into_iter().map(|x| Token::Bytes(x)).collect();

        ethabi::encode(&[
            Token::Array(operators_token),
            Token::Array(weights_token),
            Token::Uint(threshold.into()),
            Token::Array(signatures_token),
        ])
    }

    #[test]
    fn proof_encode_decode() {
        let msg_hash = H256::zero();
        let proof = &[1, 2, 3];
        let state = OperatorsState {
            hash: H256::zero(),
            epoch: u64::MAX,
        };

        // Input params
        let operators = vec![[1u8; 20], [2u8; 20]];
        let weights = vec![100, 200];
        let threshold = 99;
        let signatures = vec![vec![1], vec![2]];

        // Encode
        let encoded = encode(
            operators.clone(),
            weights.clone(),
            threshold,
            signatures.clone(),
        );

        // Now decode
        let decoded = decode(&encoded).expect("Should decode");

        if let [Token::Array(operators_token), Token::Array(weights_token), threshold_token, Token::Array(signatures_token)] =
            decoded.as_slice()
        {
            let expected_operators: Vec<Token> = operators
                .into_iter()
                .map(|x| Token::Address(x.into()))
                .collect();
            assert_eq!(operators_token.clone(), expected_operators);

            let expected_weights: Vec<Token> =
                weights.into_iter().map(|x| Token::Uint(x.into())).collect();
            assert_eq!(weights_token.clone(), expected_weights);

            let expected_threshold = Token::Uint(threshold.into());
            assert_eq!(threshold_token.clone(), expected_threshold);

            let expected_signatures: Vec<Token> =
                signatures.into_iter().map(|x| Token::Bytes(x)).collect();
            assert_eq!(signatures_token.clone(), expected_signatures);
        } else {
            panic!("Failed")
        }
    }
}
