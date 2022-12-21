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
		tokens.as_slice() {
			let operators = operators_token
				.into_iter()
				.flat_map(|x| match x {
					Token::Address(x) => Ok(x.clone()),
					_ =>  Err(ethabi::Error::InvalidData),
				})
				.collect();
			let weights = weights_token
				.into_iter()
				.flat_map(|x| match x {
					Token::Uint(w) => Ok(w.as_u128()),
					_ =>  Err(ethabi::Error::InvalidData),
				})
				.collect();
			let threshold = t.as_u128();
			let signatures = signatures_token
				.into_iter()
				.flat_map(|x| match x {
					Token::Bytes(x) => Ok(x.clone()),
					_ =>  Err(ethabi::Error::InvalidData),
				})
				.collect();

			return Ok(Proof {
				operators,
				weights,
				threshold,
				signatures
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
	).map(|x| Proof::try_from(x))?
}

#[derive(PartialEq, Debug)]
pub enum SignatureError {
	/// The signature is expected to be RSV formatted
	InvalidRSVSignature,
	/// Couldn't recover the signer from the signature
	InvalidSignature,
	/// Couldn't find the signer of a signature in the list of operators
	MalformedSigners,
	/// Not enough signatures found to meet the threshold
	LowSignaturesWeight,
}

type Success = ();

/// Verifies that the proof holds enough signatures past the threshold.
/// Fails if not enough operators signed the `msg_hash` to meet the threshold.
pub fn validate_signatures(
	msg_hash: H256,
	proof: Proof,
) -> Result<Success, SignatureError> {
	let Proof { operators, weights, threshold, signatures } = proof;
	let mut weight = 0;

	for signature in signatures.into_iter() {
		let signer = ecdsa::recover(msg_hash, signature)?;

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

pub mod ecdsa {
	use sp_core::{keccak_256, H160, H256};
	use crate::proof::SignatureError;
	use eth_encode_packed::{abi::encode_packed, SolidityDataType};

	/// Returns the address that signed a hashed message (`hash`) with
	/// `signature`. This address can then be used for verification purposes.
	/// The `signature` is expected to be formatted in the RSV format.
	pub fn recover(hash: H256, signature: Vec<u8>) -> Result<H160, SignatureError> {
		let rsv = signature
			.as_slice()
			.try_into()
			.map_err(|_| SignatureError::InvalidSignature)?;

		match sp_io::crypto::secp256k1_ecdsa_recover(&rsv, &hash.into()) {
			Ok(pubkey) => Ok(H160::from(H256::from_slice(keccak_256(&pubkey).as_slice()))),
			Err(_) => Err(SignatureError::InvalidSignature),
		}
	}

	/// Returns an Ethereum Signed Message, created from a `hash`. This replicates the behaviour of
	/// the https://github.com/ethereum/wiki/wiki/JSON-RPC#eth_sign JSON-RPC method.
	#[allow(dead_code)]
	fn to_eth_signed_message_hash(hash: [u8; 32]) -> [u8; 32] {
		let (data, _) = encode_packed(&[
			SolidityDataType::String("\x19Ethereum Signed Message:\n32"),
			SolidityDataType::Bytes(&hash.clone()),
		]);

		keccak_256(&data)
	}
}