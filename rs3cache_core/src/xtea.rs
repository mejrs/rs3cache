#![cfg(feature = "osrs")]

use std::{array, collections::HashMap, fs::File, io::BufReader, path::Path};

use serde::{Deserialize, Serialize};
use serde_json;

use crate::utils::error::CacheResult;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Xtea {
    pub mapsquare: u32,
    pub key: [i32; 4],
}

impl Xtea {
    pub fn load(path: impl AsRef<Path>) -> CacheResult<HashMap<u32, Self>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let xteas: Vec<Self> = serde_json::from_reader(reader)?;
        let map = xteas.into_iter().map(|xtea| (xtea.mapsquare, xtea)).collect();
        Ok(map)
    }
    fn decrypt_block(block: &[u8], xtea: &Xtea) -> array::IntoIter<u8, 8> {
        let v0: [u8; 4] = [block[0], block[1], block[2], block[3]];
        let mut v0 = u32::from_be_bytes(v0);

        let v1: [u8; 4] = [block[4], block[5], block[6], block[7]];
        let mut v1 = u32::from_be_bytes(v1);

        const GOLDEN_RATIO: u32 = 0x9E3779B9;
        const ROUNDS: u32 = 32;

        let mut sum = GOLDEN_RATIO.wrapping_mul(ROUNDS);
        for _ in 0..ROUNDS {
            /*let a = v0.wrapping_shl(4);
            let b = v0.wrapping_shr(5);
            let s = sum.wrapping_shr(11);*/
            v1 = v1.wrapping_sub(
                (v0.wrapping_shl(4) ^ v0.wrapping_shr(5)).wrapping_add(v0) ^ (sum.wrapping_add(xtea.key[(sum.wrapping_shr(11) & 3) as usize] as u32)),
            );

            sum = sum.wrapping_sub(GOLDEN_RATIO);
            /*
            let c = v1.wrapping_shl(4);
            let d = v1.wrapping_shr(5);*/
            v0 =
                v0.wrapping_sub((v1.wrapping_shl(4) ^ v1.wrapping_shr(5)).wrapping_add(v1) ^ (sum.wrapping_add(xtea.key[(sum & 3) as usize] as u32)));
        }

        let v0: [u8; 4] = v0.to_be_bytes();
        let v1: [u8; 4] = v1.to_be_bytes();

        array::IntoIter::new([v0[0], v0[1], v0[2], v0[3], v1[0], v1[1], v1[2], v1[3]])
    }

    pub fn decrypt(input: impl AsRef<[u8]>, xtea: Xtea) -> Vec<u8> {
        let input = input.as_ref();
        let mut iter = input.chunks_exact(8);
        let mut output: Vec<u8> = iter.by_ref().flat_map(|block| Xtea::decrypt_block(block, &xtea)).collect();

        output.extend(iter.remainder());
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm() {
        let xtea = Xtea {
            mapsquare: 12850,
            key: [-729586325, 659151050, 316388445, -2117896833],
        };

        let input = std::fs::read("tests/xtea/encrypted.dat").unwrap();

        let output = Xtea::decrypt(input, xtea);

        let should_be_output = std::fs::read("tests/xtea/decrypted.dat").unwrap();

        itertools::izip!(&output, &should_be_output).enumerate().for_each(|(count, (a, b))| {
            assert_eq!(a, b, "Mismatch at position {}.", count);
        });

        assert_eq!(output.len(), should_be_output.len());
    }
}
