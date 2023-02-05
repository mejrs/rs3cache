#![cfg(feature = "dat2")]

use std::{array, backtrace::Backtrace, collections::HashMap, fs::File, io::BufReader, path::Path};

use serde::{Deserialize, Serialize};
use serde_json;

use crate::error::{CacheError, CacheResult, Context};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Xtea {
    pub mapsquare: u32,
    pub key: [i32; 4],
}

impl Xtea {
    pub fn load(path: impl AsRef<Path>) -> CacheResult<HashMap<u32, Self>> {
        let path = path.as_ref();
        let file = File::open(path).context(path)?;
        let reader = BufReader::new(file);

        let xteas: Vec<Self> = serde_json::from_reader(reader).map_err(|cause| CacheError::xtea_load_error(cause, path.into()))?;
        let map = xteas.into_iter().map(|xtea| (xtea.mapsquare, xtea)).collect();
        Ok(map)
    }

    fn decrypt_block([a0, a1, a2, a3, b0, b1, b2, b3]: [u8; 8], xtea: &Xtea) -> [u8; 8] {
        let mut v0 = u32::from_be_bytes([a0, a1, a2, a3]);
        let mut v1 = u32::from_be_bytes([b0, b1, b2, b3]);

        const GOLDEN_RATIO: u32 = 0x9E3779B9;
        const ROUNDS: u32 = 32;

        let mut sum = GOLDEN_RATIO.wrapping_mul(ROUNDS);
        for _ in 0..ROUNDS {
            v1 = v1.wrapping_sub(
                (v0.wrapping_shl(4) ^ v0.wrapping_shr(5)).wrapping_add(v0) ^ (sum.wrapping_add(xtea.key[(sum.wrapping_shr(11) & 3) as usize] as u32)),
            );

            sum = sum.wrapping_sub(GOLDEN_RATIO);
            v0 =
                v0.wrapping_sub((v1.wrapping_shl(4) ^ v1.wrapping_shr(5)).wrapping_add(v1) ^ (sum.wrapping_add(xtea.key[(sum & 3) as usize] as u32)));
        }

        let [a0, a1, a2, a3]: [u8; 4] = v0.to_be_bytes();
        let [b0, b1, b2, b3]: [u8; 4] = v1.to_be_bytes();

        [a0, a1, a2, a3, b0, b1, b2, b3]
    }

    pub fn decrypt(input: impl AsRef<[u8]>, xtea: Xtea) -> Vec<u8> {
        let input = input.as_ref();
        let mut iter = input.array_chunks::<8>();
        let mut output: Vec<u8> = iter.by_ref().flat_map(|block| Xtea::decrypt_block(*block, &xtea)).collect();

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

        let input = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/xtea_encrypted.dat")).to_vec();

        let output = Xtea::decrypt(input, xtea);

        let should_be_output = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/xtea_decrypted.dat"));

        itertools::izip!(&output, should_be_output).enumerate().for_each(|(count, (a, b))| {
            assert_eq!(a, b, "Mismatch at position {count}.");
        });

        assert_eq!(output.len(), should_be_output.len());
    }
}
