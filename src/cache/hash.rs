pub fn hash_djb2(s: impl AsRef<[u8]>) -> i32 {
    let mut n: i32 = 0;
    for x in s.as_ref() {
        let x = *x as i32;
        n = x .wrapping_add(n.wrapping_shl(5)).wrapping_sub(n);
    }
    n
}
