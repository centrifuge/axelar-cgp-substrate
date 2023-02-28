use sp_std::cmp::min;

// Taken from: https://github.com/centrifuge/centrifuge-chain/blob/3161e6d547e60096f867ecc4fa954a5c97513ce5/libs/utils/src/lib.rs#L21
pub fn vec_to_fixed_array<const S: usize>(src: Vec<u8>) -> [u8; S] {
    let mut dest = [0; S];
    let len = min(src.len(), S);
    dest[..len].copy_from_slice(&src.as_slice()[..len]);

    dest
}
