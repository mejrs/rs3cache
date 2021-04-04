pub fn test_function() {
    use crate::cache::index::*;

    let c = CacheIndex::new(8).unwrap();
    let m = c.metadatas();
    for metadata in m.iter() {
        println!("{:?}", metadata);
    }
}
