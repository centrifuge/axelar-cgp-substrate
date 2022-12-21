use eth_encode_packed::{abi::encode_packed, SolidityDataType};
use sp_core::{keccak_256, H160, H256};
use sp_io::EcdsaVerifyError;

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
