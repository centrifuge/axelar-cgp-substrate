// Copyright 2023 Centrifuge Foundation (centrifuge.io).
//
// This file is part of the axelar-cgp-substrate project.
// Axelar-cgp-substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version (see http://www.gnu.org/licenses).
// Centrifuge is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// Ensure we're `no_std` when compiling for WebAssembly.
#![cfg_attr(not(feature = "std"), no_std)]

use eth_encode_packed::{abi::encode_packed, SolidityDataType};
use libsecp256k1::{sign, Message, PublicKey, SecretKey};
use sp_core::{keccak_256, sp_std, H160, H256};
use sp_io::EcdsaVerifyError;
use sp_std::vec::Vec;

/// Returns the address that signed a hashed message (`hash`) with
/// `signature`. This address can then be used for verification purposes.
/// The `signature` is expected to be formatted in the RSV format.
pub fn recover(hash: H256, signature: Vec<u8>) -> Result<H160, EcdsaVerifyError> {
	let rsv: [u8; 65] = signature
		.as_slice()
		.try_into()
		.map_err(|_| EcdsaVerifyError::BadSignature)?;
	let pubkey = sp_io::crypto::secp256k1_ecdsa_recover(&rsv, &hash.into())?;

	Ok(H160::from(H256::from_slice(keccak_256(&pubkey).as_slice())))
}

/// Returns Signature (r,s,v) concatenated
#[allow(dead_code)]
#[cfg(feature = "std")]
pub fn sign_message(msg: H256, secret: &[u8; 32]) -> Vec<u8> {
	let sec_key = SecretKey::parse(secret);
	let message = Message::parse(msg.as_fixed_bytes());
	let (sig, rec_id) = sign(&message, &sec_key.unwrap());

	let mut rsv = sig.serialize().to_vec();
	rsv.push(rec_id.serialize());

	rsv
}

/// Generates random Secp256k1 key pair
#[allow(dead_code)]
#[cfg(feature = "std")]
pub fn generate_keypair() -> (Vec<u8>, [u8; 32]) {
	let secret = SecretKey::random(&mut rand::thread_rng());
	let public = PublicKey::from_secret_key(&secret);

	(public.serialize()[1..65].to_vec(), secret.serialize())
}

/// Returns an Ethereum Signed Message, created from a `hash`. This replicates the behaviour of
/// the https://github.com/ethereum/wiki/wiki/JSON-RPC#eth_sign JSON-RPC method.
#[allow(dead_code)]
#[cfg(feature = "std")]
pub fn to_eth_signed_message_hash(hash: [u8; 32]) -> [u8; 32] {
	let (data, _) = encode_packed(&[
		SolidityDataType::String("\x19Ethereum Signed Message:\n32"),
		SolidityDataType::Bytes(&hash.clone()),
	]);

	keccak_256(&data)
}
