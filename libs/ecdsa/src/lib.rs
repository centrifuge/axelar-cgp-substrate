use eth_encode_packed::{abi::encode_packed, SolidityDataType};
use libsecp256k1::{
    recover as recover2, sign, Message, PublicKey, RecoveryId, SecretKey, Signature,
};
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

/// Returns Signature (r,s,v) concatenated
#[allow(dead_code)]
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
pub fn generate_keypair() -> (Vec<u8>, [u8; 32]) {
    let secret = SecretKey::random(&mut rand::thread_rng());
    let public = PublicKey::from_secret_key(&secret);

    (public.serialize()[1..65].to_vec(), secret.serialize())
}

/// Returns an Ethereum Signed Message, created from a `hash`. This replicates the behaviour of
/// the https://github.com/ethereum/wiki/wiki/JSON-RPC#eth_sign JSON-RPC method.
#[allow(dead_code)]
pub fn to_eth_signed_message_hash(hash: [u8; 32]) -> [u8; 32] {
    let (data, _) = encode_packed(&[
        SolidityDataType::String("\x19Ethereum Signed Message:\n32"),
        SolidityDataType::Bytes(&hash.clone()),
    ]);

    keccak_256(&data)
}
