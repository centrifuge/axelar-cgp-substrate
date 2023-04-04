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

use sp_std::cmp::min;

// Taken from: https://github.com/centrifuge/centrifuge-chain/blob/3161e6d547e60096f867ecc4fa954a5c97513ce5/libs/utils/src/lib.rs#L21
pub fn vec_to_fixed_array<const S: usize>(src: Vec<u8>) -> [u8; S] {
    let mut dest = [0; S];
    let len = min(src.len(), S);
    dest[..len].copy_from_slice(&src.as_slice()[..len]);

    dest
}

pub mod proofs {
    use ethabi::{ParamType, Token};

    /// Test utils function that encodes the data of a proof to ethabi::Bytes
    pub fn encode(
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

    /// Test utils function to decode the input of a `execute` message, expected to contain a
    /// bytearray with the data of the call to be executed and a bytearray containing the proof.
    pub fn decode_input(input: &[u8]) -> Result<(Vec<u8>, Vec<u8>), ethabi::Error> {
        let res = ethabi::decode(
            &[
                // data
                ParamType::Bytes,
                // proof
                ParamType::Bytes,
            ],
            input,
        );

        match res {
            Ok(params) => match params.as_slice() {
                [Token::Bytes(data), Token::Bytes(proof)] => Ok((data.clone(), proof.clone())),
                _ => panic!("todo(nuno): wrong input"),
            },
            _ => panic!("todo(nuno): failed to decode input"),
        }
    }
}
