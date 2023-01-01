use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    env::{self, VarError},
    fs::{self, File},
    io::{self, BufReader, Cursor, Read, Seek, SeekFrom, Write},
    marker::PhantomData,
    ops::RangeInclusive,
    path::{Path, PathBuf},
    sync::Arc,
};

use bytes::{Buf, Bytes};
use itertools::iproduct;
use path_macro::path;

use crate::{
    arc::Archive,
    buf::{BufExtra, ReadError},
    decoder,
    error::{CacheError, CacheErrorKind, CacheResult},
    index::{CacheIndex, CachePath, IndexState, Initial},
    meta::{IndexMetadata, Metadata},
    xtea::Xtea,
};
impl<S> CacheIndex<S>
where
    S: IndexState,
{
    fn get_entry(a: u32, b: u32, path: &Arc<CachePath>) -> CacheResult<(u32, u32)> {
        let file = path!(**path / "cache" / format!("main_file_cache.idx{a}"));
        let entry_data = fs::read(&file).map_err(|e| CacheError::cache_not_found(e, file, path.clone()))?;
        let mut buf = Cursor::new(entry_data);
        buf.seek(SeekFrom::Start((b * 6) as _)).unwrap();
        Ok((buf.try_get_uint(3)? as u32, buf.try_get_uint(3)? as u32))
    }

    fn read_index(&self, a: u32, b: u32) -> CacheResult<Vec<u8>> {
        let mut buffer = BufReader::new(&self.file);

        let (length, mut sector) = Self::get_entry(a, b, &self.path)?;

        let mut read_count = 0;
        let mut part = 0;
        let mut data = Vec::with_capacity(length as _);

        while sector != 0 {
            buffer.seek(SeekFrom::Start((sector * 520) as _)).map_err(|_| ReadError::eof())?;
            let (_header_size, current_archive, block_size) = if b >= 0xFFFF {
                let mut buf = [0; 4];
                buffer.read_exact(&mut buf).map_err(|_| ReadError::eof())?;
                (10, i32::from_be_bytes(buf), 510.min(length - read_count))
            } else {
                let mut buf = [0; 2];
                buffer.read_exact(&mut buf).map_err(|_| ReadError::eof())?;
                (8, u16::from_be_bytes(buf) as _, 512.min(length - read_count))
            };

            let current_part = {
                let mut buf = [0; 2];
                buffer.read_exact(&mut buf).map_err(|_| ReadError::eof())?;
                u16::from_be_bytes(buf)
            };
            let new_sector = {
                let mut buf = [0; 4];
                buffer.read_exact(&mut buf[1..4]).map_err(|_| ReadError::eof())?;
                u32::from_be_bytes(buf)
            };
            let current_index = {
                let mut buf = [0; 1];
                buffer.read_exact(&mut buf).map_err(|_| ReadError::eof())?;
                u8::from_be_bytes(buf)
            };

            assert_eq!(a, current_index as u32);
            assert_eq!(b, current_archive as u32);
            assert_eq!(part, current_part as u32);

            part += 1;
            read_count += block_size;
            sector = new_sector;

            let mut buf = [0u8; 512];
            buffer.read_exact(&mut buf[..(block_size as usize)]).map_err(|_| ReadError::eof())?;

            data.extend_from_slice(&buf[..(block_size as usize)]);
        }
        Ok(data)
    }

    pub fn get_file(&self, metadata: &Metadata) -> CacheResult<Bytes> {
        let data = self.read_index(metadata.index_id(), metadata.archive_id())?;
        Ok(decoder::decompress(data, None)?)
    }

    pub fn xteas(&self) -> &Option<HashMap<u32, Xtea>> {
        &self.xteas
    }

    pub fn archive_with_xtea(&self, archive_id: u32, xtea: Option<Xtea>) -> CacheResult<Archive> {
        let metadata = self
            .metadatas()
            .get(&archive_id)
            .ok_or_else(|| CacheError::archive_missing(self.index_id(), archive_id))?;
        let data = self.read_index(metadata.index_id(), metadata.archive_id())?;
        let data = decoder::decompress(data, xtea)?;
        Ok(Archive::deserialize(metadata, data))
    }

    pub fn archive_by_name(&self, name: String) -> CacheResult<Bytes> {
        let hash = crate::hash::hash_djb2(&name);
        for (_, m) in self.metadatas.iter() {
            if m.name() == Some(hash) {
                return self.get_file(m);
            }
        }
        Err(CacheError::archive_missing(0, 0))
    }
}

impl CacheIndex<Initial> {
    /// Constructor for [`CacheIndex`].
    ///
    /// # Errors
    ///
    /// Raises [`CacheNotFoundError`](CacheError::CacheNotFoundError) if the cache database cannot be found.
    pub fn new(index_id: u32, path: Arc<CachePath>) -> CacheResult<CacheIndex<Initial>> {
        let file = path!(path.as_ref() / "cache" / "main_file_cache.dat2");

        let file = match File::open(&file) {
            Ok(f) => f,
            Err(e) => return Err(CacheError::cache_not_found(e, file, path)),
        };
        let xteas = if index_id == 5 {
            let path = path!(&*path / "xteas.json");

            // Try to load either xteas.json or keys.json
            match Xtea::load(&path) {
                Ok(file) => Some(file),
                // Let's try looking somewhere else
                Err(e1) if let CacheErrorKind::IoError(cause, _) = e1.kind() && cause.kind() == io::ErrorKind::NotFound => {
                    let alt_path = path!(&*path / "keys.json");
                    match Xtea::load(alt_path){
                        Ok(file) => Some(file),
                        Err(_) => return Err(e1)
                    }
                }
                Err(other) => return Err(other),
            }
        } else {
            None
        };

        // `s` is in a partially initialized state here
        let mut s = Self {
            path,
            index_id,
            metadatas: IndexMetadata::empty(),
            file,
            xteas,
            state: Initial {},
        };

        let metadatas = {
            let data = s.read_index(255, index_id)?;
            let data = decoder::decompress(data, None)?;
            IndexMetadata::deserialize(index_id, data)?
        };

        s.metadatas = metadatas;
        // `s` is now fully initialized

        Ok(s)
    }
}
