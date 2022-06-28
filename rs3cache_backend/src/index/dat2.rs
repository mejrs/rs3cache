use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    env::{self, VarError},
    fs::{self, File},
    io::{self, BufReader, Cursor, Read, Seek, SeekFrom, Write},
    marker::PhantomData,
    ops::RangeInclusive,
    path::{Path, PathBuf},
};

use bytes::{Buf, Bytes};
use itertools::iproduct;
use path_macro::path;

use crate::{
    arc::Archive,
    buf::BufExtra,
    decoder,
    error::{CacheError, CacheResult},
    index::{CacheIndex, IndexState, Initial},
    meta::{IndexMetadata, Metadata},
    xtea::Xtea,
};

impl<S> CacheIndex<S>
where
    S: IndexState,
{
    fn get_entry(a: u32, b: u32, folder: impl AsRef<Path>) -> CacheResult<(u32, u32)> {
        let file = path!(folder / "cache" / format!("main_file_cache.idx{a}"));
        let entry_data = fs::read(&file).map_err(|e| CacheError::CacheNotFoundError(e, file))?;
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
            buffer.seek(SeekFrom::Start((sector * 520) as _))?;
            let (_header_size, current_archive, block_size) = if b >= 0xFFFF {
                let mut buf = [0; 4];
                buffer.read_exact(&mut buf)?;
                (10, i32::from_be_bytes(buf), 510.min(length - read_count))
            } else {
                let mut buf = [0; 2];
                buffer.read_exact(&mut buf)?;
                (8, u16::from_be_bytes(buf) as _, 512.min(length - read_count))
            };

            let current_part = {
                let mut buf = [0; 2];
                buffer.read_exact(&mut buf)?;
                u16::from_be_bytes(buf)
            };
            let new_sector = {
                let mut buf = [0; 4];
                buffer.read_exact(&mut buf[1..4])?;
                u32::from_be_bytes(buf)
            };
            let current_index = {
                let mut buf = [0; 1];
                buffer.read_exact(&mut buf)?;
                u8::from_be_bytes(buf)
            };

            assert_eq!(a, current_index as u32);
            assert_eq!(b, current_archive as u32);
            assert_eq!(part, current_part as u32);

            part += 1;
            read_count += block_size;
            sector = new_sector;

            let mut buf = [0u8; 512];
            buffer.read_exact(&mut buf[..(block_size as usize)])?;

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
            .ok_or_else(|| CacheError::ArchiveNotFoundError(self.index_id(), archive_id))?;
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
        Err(CacheError::ArchiveNotFoundError(0, 0))
    }
}

impl CacheIndex<Initial> {
    /// Constructor for [`CacheIndex`].
    ///
    /// # Errors
    ///
    /// Raises [`CacheNotFoundError`](CacheError::CacheNotFoundError) if the cache database cannot be found.
    pub fn new(index_id: u32, folder: impl AsRef<Path>) -> CacheResult<CacheIndex<Initial>> {
        let file = path!(folder / "cache" / "main_file_cache.dat2");

        let file = File::open(&file).map_err(|e| CacheError::CacheNotFoundError(e, file))?;
        let xteas = if index_id == 5 {
            let path = path!(folder / "xteas.json");

            // Try to laod either xteas.json or keys.json
            match Xtea::load(&path) {
                Ok(file) => Some(file),
                Err(CacheError::IoError(e1)) if e1.kind() == io::ErrorKind::NotFound => {
                    let alt_path = path!(folder / "keys.json");
                    Some(Xtea::load(&alt_path).map_err(|e2| {
                        let path = path.to_string_lossy();
                        let alt_path = alt_path.to_string_lossy();
                        let e1 = e1.to_string();
                        let e2 = e2.to_string();
                        io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("Cannot to find xtea keys at either {path} or {alt_path}: {e1} \n {e2}"),
                        )
                    })?)
                }
                Err(other) => return Err(other),
            }
        } else {
            None
        };

        // `s` is in a partially initialized state here
        let mut s = Self {
            path: folder.as_ref().to_path_buf(),
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
