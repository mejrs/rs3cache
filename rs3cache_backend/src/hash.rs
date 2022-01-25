pub fn hash_djb2(s: impl AsRef<[u8]>) -> i32 {
    let mut n: i32 = 0;
    for x in s.as_ref() {
        let x = *x as i32;
        n = x.wrapping_add(n.wrapping_shl(5)).wrapping_sub(n);
    }
    n
}

#[cfg(feature = "dat")]
pub fn hash_archive(s: &str) -> i32 {
    let mut n: i32 = 0;
    for x in s.chars() {
        assert!(x.is_ascii());
        let x = x.to_ascii_uppercase() as i32;
        n = n.wrapping_mul(61).wrapping_add(x).wrapping_add(-32);
    }
    n
}

#[cfg(all(test, feature = "dat"))]
mod legacy {
    use super::*;

    #[test]
    fn test_hash1() {
        let name = "loc.dat";

        let hash = hash_archive(name);
        assert_eq!(hash, 682978269);
    }

    #[test]
    fn test_hash2() {
        let name = "obj.dat";

        let hash = hash_archive(name);
        assert_eq!(hash, -1667617738);
    }

    #[test]
    fn test_hash3() {
        let name = "map_version";

        let hash = hash_archive(name);
        assert_eq!(hash, -923525801);
    }
}
