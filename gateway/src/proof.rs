//! # Axelar CGP
//!
//! This pallet implements the Axelar Cross-Chain Gateway Protocol

// Ensure we're `no_std` when compiling for WebAssembly.
#![cfg_attr(not(feature = "std"), no_std)]
use ethabi::{Address, ParamType, Token};
use sp_core::H256;

#[derive(PartialEq, Debug)]
pub struct Proof {
    pub operators: Vec<Address>,
    pub weights: Vec<u128>,
    pub threshold: u128,
    pub signatures: Vec<Vec<u8>>,
}

impl TryFrom<Vec<Token>> for Proof {
    type Error = ethabi::Error;

    fn try_from(tokens: Vec<Token>) -> Result<Self, Self::Error> {
        if let [Token::Array(operators_token), Token::Array(weights_token), Token::Uint(t), Token::Array(signatures_token)] =
            tokens.as_slice()
        {
            let operators = operators_token
                .into_iter()
                .flat_map(|x| match x {
                    Token::Address(x) => Ok(x.clone()),
                    _ => Err(ethabi::Error::InvalidData),
                })
                .collect();
            let weights = weights_token
                .into_iter()
                .flat_map(|x| match x {
                    Token::Uint(w) => Ok(w.as_u128()),
                    _ => Err(ethabi::Error::InvalidData),
                })
                .collect();
            let threshold = t.as_u128();
            let signatures = signatures_token
                .into_iter()
                .flat_map(|x| match x {
                    Token::Bytes(x) => Ok(x.clone()),
                    _ => Err(ethabi::Error::InvalidData),
                })
                .collect();

            return Ok(Proof {
                operators,
                weights,
                threshold,
                signatures,
            });
        }

        Err(ethabi::Error::InvalidData)
    }
}

/// Decode a payload expected to contain a `Proof`.
#[allow(dead_code)]
fn decode(payload: &[u8]) -> Result<Proof, ethabi::Error> {
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
    .map(|x| Proof::try_from(x))?
}

#[derive(PartialEq, Debug)]
pub enum SignatureError {
    /// The signature is invalid
    InvalidSignature,
    /// Couldn't find the signer of a signature in the list of operators
    MalformedSigners,
    /// Not enough signatures found to meet the threshold
    LowSignaturesWeight,
}

type Success = ();

/// Verifies that the proof holds enough signatures past the threshold.
/// Fails if not enough operators signed the `msg_hash` to meet the threshold.
pub fn validate_signatures(msg_hash: H256, proof: Proof) -> Result<Success, SignatureError> {
    let Proof {
        operators,
        weights,
        threshold,
        signatures,
    } = proof;
    let mut weight = 0;

    for signature in signatures.into_iter() {
        let signer = ecdsa_utils::recover(msg_hash, signature).map_err(|_| SignatureError::InvalidSignature)?;

        let index = operators
            .iter()
            .position(|x| x.0 == signer.0)
            .ok_or(SignatureError::MalformedSigners)?;

        weight += weights[index];

        if weight >= threshold {
            return Ok(());
        }
    }

    Err(SignatureError::LowSignaturesWeight)
}
