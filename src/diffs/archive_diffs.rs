//use crate::cache::{index::CacheIndex, indextype::IndexType, meta::IndexMetadata};

// todo: make this use dates properly. I can't be fucked to figure out date libraries.

use crate::utils::error::CacheResult;
//use chrono::{TimeZone, Utc};
//use std::time::{SystemTime, UNIX_EPOCH};
/// Checks which archives are less that two weeks old.
pub fn diff() -> CacheResult<()> {
    /*let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time has reversed").as_secs() as i32;
    let elapsed = now - 2 * 7 * 24 * 60 * 60;

    let metadatas = CacheIndex::new(IndexType::MAPSV2)?.metadatas();
    for (key, metadata) in metadatas.iter() {
        let utc = metadata.version();
        if utc > elapsed && metadata.child_count() > 1 {
            let date = Utc.timestamp(utc.into(), 0);
            let i = key & 0x7F;
            let j = key >> 7;
            println!("Mapsquare {}, {} has last been updated at {:?}.", i, j, date)
        }
    }
    */
    Ok(())
}
