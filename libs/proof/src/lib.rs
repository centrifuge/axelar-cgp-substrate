//! # Axelar Cross-Chain Gateway Protocol
//!
//! This pallet implements the proof validation logic of the Axelar CGP.
//!
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::vec::Vec;

// ----------------------------------------------------------------------------
// Custom types and type aliases
// ----------------------------------------------------------------------------
type Bytes = Vec<u8>;
type Bytes32 = [u8; 32];

pub struct OperatorsState {
    pub hash: Bytes32,
    //todo(nuno): epoch is unit256 in solidity
    pub epoch: u128,
}

/// @dev This function takes messageHash and proof data and reverts if proof is invalid
/// @return True if provided operators are the current ones
/// Original implementation in Solidity:
/// https://github.com/axelarnetwork/axelar-cgp-solidity/blob/main/contracts/auth/AxelarAuthWeighted.sol#L28
pub fn validate(_msg_hash: Bytes32, _proof: Bytes, _state: OperatorsState) -> bool {
    // (address[] memory operators, uint256[] memory weights, uint256 threshold, bytes[] memory signatures) = abi.decode(
    //     proof,
    //     (address[], uint256[], uint256, bytes[])
    // );
    //
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

// ----------------------------------------------------------------------------
// Tests
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use sp_std::vec;

    #[test]
    fn it_works() {
        let msg_hash = [0u8; 32];
        let proof = vec![1, 2, 3];
        let state = OperatorsState {
            hash: [1u8; 32],
            epoch: u128::MAX,
        };

        assert_eq!(validate(msg_hash, proof, state), false)
    }
}
