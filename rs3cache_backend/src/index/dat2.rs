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

use ::error::Context;
use bytes::{Buf, Bytes};
use itertools::iproduct;
use path_macro::path;

use crate::{
    arc::Archive,
    buf::{BufExtra, FileSeek, ReadError},
    decoder,
    error::{self, CacheError, CacheResult, CannotOpen},
    index::*,
    meta::{IndexMetadata, Metadata},
    xtea::Xtea,
};
impl<S> CacheIndex<S>
where
    S: IndexState,
{
    fn get_entry(a: u32, b: u32, input: &CachePath) -> CacheResult<(u32, u32)> {
        let file = path!(input / "cache" / format!("main_file_cache.idx{a}"));
        let entry_data = fs::read(&file).context(CannotOpen { file, input: input.clone() })?;
        let mut buf = Cursor::new(entry_data);
        buf.seek(SeekFrom::Start((b * 6) as _)).unwrap();
        Ok((
            buf.try_get_uint(3).context(error::Read { what: "cache entries" })? as u32,
            buf.try_get_uint(3).context(error::Read { what: "cache entries" })? as u32,
        ))
    }

    fn read_index(&self, a: u32, b: u32) -> CacheResult<Vec<u8>> {
        let mut buffer = BufReader::new(&self.file);

        let (length, mut sector) = Self::get_entry(a, b, &self.input)?;

        let mut read_count = 0;
        let mut part = 0;
        let mut data = Vec::with_capacity(length as _);

        while sector != 0 {
            buffer.seek(SeekFrom::Start((sector * 520) as _)).context(FileSeek).context(error::Read {
                what: "buffer start position",
            })?;
            let (_header_size, current_archive, block_size) = if b >= 0xFFFF {
                let mut buf = [0; 4];
                buffer
                    .read_exact(&mut buf)
                    .context(FileSeek)
                    .context(error::Read { what: "archive checksum" })?;
                (10, i32::from_be_bytes(buf), 510.min(length - read_count))
            } else {
                let mut buf = [0; 2];
                buffer
                    .read_exact(&mut buf)
                    .context(FileSeek)
                    .context(error::Read { what: "archive checksum" })?;
                (8, u16::from_be_bytes(buf) as _, 512.min(length - read_count))
            };

            let current_part = {
                let mut buf = [0; 2];
                buffer
                    .read_exact(&mut buf)
                    .context(FileSeek)
                    .context(error::Read { what: "part checksum" })?;
                u16::from_be_bytes(buf)
            };
            let new_sector = {
                let mut buf = [0; 4];
                buffer.read_exact(&mut buf[1..4]).context(FileSeek).context(error::Read {
                    what: "next sector position",
                })?;
                u32::from_be_bytes(buf)
            };
            let current_index = {
                let mut buf = [0; 1];
                buffer.read_exact(&mut buf).context(FileSeek).context(error::Read {
                    what: "current position checksum",
                })?;
                u8::from_be_bytes(buf)
            };

            assert_eq!(a, current_index as u32);
            assert_eq!(b, current_archive as u32);
            assert_eq!(part, current_part as u32);

            part += 1;
            read_count += block_size;
            sector = new_sector;

            let mut buf = [0u8; 512];
            buffer
                .read_exact(&mut buf[..(block_size as usize)])
                .context(FileSeek)
                .context(error::Read { what: "archive data" })?;

            data.extend_from_slice(&buf[..(block_size as usize)]);
        }
        Ok(data)
    }

    pub fn get_file(&self, metadata: &Metadata) -> CacheResult<Bytes> {
        let data = self.read_index(metadata.index_id(), metadata.archive_id())?;
        decoder::decompress(data, None).context(error::Decode)
    }

    pub fn xteas(&self) -> &Option<HashMap<u32, Xtea>> {
        &self.xteas
    }

    pub fn archive_with_xtea(&self, archive_id: u32, xtea: Option<Xtea>) -> CacheResult<Archive> {
        let metadata = self
            .metadatas()
            .get(&archive_id)
            .context(ArchiveMissing {
                index_id: self.index_id,
                archive_id,
            })
            .context(error::Integrity)?;
        let data = self.read_index(metadata.index_id(), metadata.archive_id())?;
        let data = decoder::decompress(data, xtea).context(error::Decode)?;
        Ok(Archive::deserialize(metadata, data))
    }

    pub fn archive_by_name(&self, name: String) -> CacheResult<Bytes> {
        let hash = crate::hash::hash_djb2(&name);
        for (_, m) in self.metadatas.iter() {
            if m.name() == Some(hash) {
                return self.get_file(m);
            }
        }
        Err(ArchiveMissingNamed::new(self.index_id, name)).context(error::Integrity)
    }
}

impl CacheIndex<Initial> {
    /// Constructor for [`CacheIndex`].
    ///
    /// # Errors
    ///
    /// Raises [`CacheNotFoundError`](CacheError::CacheNotFoundError) if the cache database cannot be found.
    pub fn new(index_id: u32, input: CachePath) -> CacheResult<CacheIndex<Initial>> {
        let file = path!(input.as_ref() / "cache" / "main_file_cache.dat2");

        let file = File::open(&file).with_context(|| CannotOpen {
            file: file.clone(),
            input: input.clone(),
        })?;
        let xteas = if index_id == 5 {
            let path1 = path!(input / "xteas.json");

            // Try to load either xteas.json or keys.json
            match Xtea::load(path1) {
                Ok(file) => Some(file),
                // Let's try looking somewhere else
                Err(_) => {
                    let alt_path = path!(input / "keys.json");
                    match Xtea::load(alt_path) {
                        Ok(file) => Some(file),
                        Err(e) => return Err(e),
                    }
                }
            }
        } else {
            None
        };

        // `s` is in a partially initialized state here
        let mut s = Self {
            input,
            index_id,
            metadatas: IndexMetadata::empty(),
            file,
            xteas,
            state: Initial {},
        };

        let metadatas = {
            let data = s.read_index(255, index_id)?;
            let data = decoder::decompress(data, None).context(error::Decode)?;
            IndexMetadata::deserialize(index_id, data).context(error::Read { what: "index metadata" })?
        };

        s.metadatas = metadatas;
        // `s` is now fully initialized

        Ok(s)
    }
}
