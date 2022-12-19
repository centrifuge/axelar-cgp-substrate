//! # Axelar Cross-Chain Gateway Protocol
//!
//! This pallet implements the proof validation logic of the Axelar CGP.
//!
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::boxed::Box;
// use ethabi::ParamType::Address;
use ethabi::ethereum_types::H512;
use ethabi::{Address, ParamType};

use sp_core::H256;
use sp_io::hashing::keccak_256;
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

// https://github.com/axelarnetwork/axelar-cgp-solidity/blob/main/contracts/auth/AxelarAuthWeighted.sol#L88
// TODO(nuno): given that `operators`, `signatures`, and maybe `weight` should be sorted, I guess
// we could zip the three and pair the computation together instead of the current expensive lookups.
pub fn validate_signatures(
    msg_hash: H256,
    signatures: Vec<Vec<u8>>,
    operators: Vec<Address>,
    weights: Vec<u128>,
    threshold: u128,
) -> bool {
    let mut weight = 0;

    for s in signatures.into_iter() {
        let rsv = to_rsv(s).expect("Todo(nuno): handle");

        let res = sp_io::crypto::secp256k1_ecdsa_recover(&rsv, &msg_hash.into());
        let signer: [u8; 64] = match res {
            Ok(signer) => signer,
            Err(_) => return false,
        };

        // Hack - On Ethereum there's a ecrecover function that returns an address given a rsv
        // signature. We need something alike here.
        let addr: [u8; 20] = signer.as_slice().try_into().expect("Todo(nuno)");

        let index = operators
            .iter()
            .position(|o| o == &Address::from(addr))
            .expect("todo(nuno)");

        weight += weights[index];

        if weight >= threshold {
            return true;
        }
    }

    // In Solidity:
    // if weight sum below threshold
    // revert LowSignaturesWeight();
    false
}

fn to_rsv(signature: Vec<u8>) -> Result<[u8; 65], ()> {
    let src: [u8; 64] = signature.as_slice().try_into().map_err(|_| ())?;

    // Build the `sig` which is of type [0u8; 65]. sig is passed in RSV format. V should be either 0/1 or 27/28.
    let mut rsv = [0u8; 65];
    rsv[0..32].copy_from_slice(&src[..32]);
    rsv[32..64].copy_from_slice(&src[..64]);
    rsv[64] = 27;

    Ok(rsv)
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

    #[test]
    fn test_validate_signatures() {
        let msg_hash = H256::zero();
        let signatures = vec![vec![1; 64], vec![2; 64], vec![3; 64]];
        let operators: Vec<Address> = vec![[1u8; 20].into(), [2u8; 20].into(), [3u8; 20].into()];
        let weights: Vec<u128> = vec![1, 2, 3];
        let threshold = 2;

        assert!(validate_signatures(
            msg_hash, signatures, operators, weights, threshold
        ));
    }
}
