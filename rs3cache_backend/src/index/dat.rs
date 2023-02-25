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
    buf::{BufExtra, ReadError},
    decoder,
    error::{self, CacheError, CacheResult},
    index::*,
    meta::{IndexMetadata, Metadata},
};
impl<S> CacheIndex<S>
where
    S: IndexState,
{
    fn get_entry(a: u32, b: u32, input: &Arc<CachePath>) -> CacheResult<(u32, u32)> {
        let file = path!(**input / "cache" / format!("main_file_cache.idx{a}"));
        let entry_data = fs::read(&file)
            .context(CannotOpen { file, input: input.clone() })
            .context(error::Integrity)?;

        let mut buf = Cursor::new(entry_data);
        buf.seek(SeekFrom::Start((b * 6) as _))
            .map_err(|_| ReadError::eof())
            .context(error::Read)?;
        Ok((
            buf.try_get_uint(3).context(error::Read)? as u32,
            buf.try_get_uint(3).context(error::Read)? as u32,
        ))
    }

    fn read_index(&self, a: u32, b: u32) -> CacheResult<Vec<u8>> {
        let mut buffer = BufReader::new(&self.file);

        let (length, mut sector) = Self::get_entry(a, b, &self.input)?;

        let mut read_count = 0;
        let mut part = 0;
        let mut data = Vec::with_capacity(length as _);

        while sector != 0 {
            buffer
                .seek(SeekFrom::Start((sector * 520) as _))
                .map_err(|_| ReadError::eof())
                .context(error::Read)?;
            let (_header_size, current_archive, block_size) = if b >= 0xFFFF {
                let mut buf = [0; 4];
                buffer.read_exact(&mut buf).map_err(|_| ReadError::eof()).context(error::Read)?;
                (10, i32::from_be_bytes(buf), 510.min(length - read_count))
            } else {
                let mut buf = [0; 2];
                buffer.read_exact(&mut buf).map_err(|_| ReadError::eof()).context(error::Read)?;
                (8, u16::from_be_bytes(buf) as _, 512.min(length - read_count))
            };

            let current_part = {
                let mut buf = [0; 2];
                buffer.read_exact(&mut buf).map_err(|_| ReadError::eof()).context(error::Read)?;
                u16::from_be_bytes(buf)
            };
            let new_sector = {
                let mut buf = [0; 4];
                buffer.read_exact(&mut buf[1..4]).map_err(|_| ReadError::eof()).context(error::Read)?;
                u32::from_be_bytes(buf)
            };
            let _current_index = {
                let mut buf = [0; 1];
                buffer.read_exact(&mut buf).map_err(|_| ReadError::eof()).context(error::Read)?;
                u8::from_be_bytes(buf)
            };

            assert_eq!(b, current_archive as u32);
            assert_eq!(part, current_part as u32);

            part += 1;
            read_count += block_size;
            sector = new_sector;

            let mut buf = [0u8; 512];
            buffer
                .read_exact(&mut buf[..(block_size as usize)])
                .map_err(|_| ReadError::eof())
                .context(error::Read)?;

            data.extend_from_slice(&buf[..(block_size as usize)]);
        }
        Ok(data)
    }

    pub fn get_file(&self, metadata: &Metadata) -> CacheResult<Bytes> {
        let data = self.read_index(metadata.index_id(), metadata.archive_id())?;
        if metadata.index_id() == 0 {
            // The caller of this function is responsible for unpacking the .jag format
            return Ok(Bytes::from(data));
        }
        decoder::decompress(data).context(error::Decode)
    }

    pub fn archive_by_name(&self, name: String) -> CacheResult<Bytes> {
        let hash = crate::hash::hash_archive(&name);
        for (_, m) in self.metadatas.iter() {
            if m.name() == Some(hash) {
                return self.get_file(m);
            }
        }
        Err(IntegrityError::ArchiveMissingNamed {
            index_id: self.index_id,
            name,
        })
        .context(error::Integrity)
    }

    pub fn get_index(&mut self) -> BTreeMap<(u8, u8), MapsquareMeta> {
        let index_name = match self.index_id {
            /*
            1 => "model",
            2 => "anim",
            3 => "midi",
            */
            4 => "map",
            other => unimplemented!("getting index metadata for {other} is not supported"),
        };

        let temp = self.index_id;

        // Temporarily set the id to 0
        self.index_id = 0;
        let a = self.archive(5).unwrap();
        let mut index = a.file_named(format!("{index_name}_index")).unwrap();
        let _versions = a.file_named(format!("{index_name}_version")).unwrap();
        let _crcs = a.file_named(format!("{index_name}_crc")).unwrap();

        // Restore the index id
        self.index_id = temp;

        let mut map = BTreeMap::new();

        for _ in 0..(index.len() / 7) {
            let meta = MapsquareMeta {
                mapsquare: index.get_u16(),
                mapfile: index.get_u16(),
                locfile: index.get_u16(),
                f2p: index.get_u8() != 0,
            };
            let i = (meta.mapsquare >> 8).try_into().unwrap();
            let j = (meta.mapsquare & 0xFF) as u8;

            map.insert((i, j), meta);
        }

        map
    }
}

impl CacheIndex<Initial> {
    /// Constructor for [`CacheIndex`].
    ///
    /// # Errors
    ///
    /// Raises [`CacheNotFoundError`](CacheError::CacheNotFoundError) if the cache database cannot be found.
    pub fn new(index_id: u32, input: Arc<CachePath>) -> CacheResult<CacheIndex<Initial>> {
        let file = path!(&*input / "cache/main_file_cache.dat");

        let file = File::open(&file)
            .context(CannotOpen { file, input: input.clone() })
            .context(error::Integrity)?;

        Ok(Self {
            input,
            index_id,
            metadatas: IndexMetadata::empty(),
            file,
            state: Initial {},
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MapsquareMeta {
    pub mapsquare: u16,
    pub mapfile: u16,
    pub locfile: u16,
    pub f2p: bool,
}
/*
#[derive(::error::Error, Debug)]
pub enum IntegrityError {
    #[error = "cannot open cache"]
    CannotOpen {
        #[source]
        source: std::io::Error,
        path: PathBuf,
        input: Arc<CachePath>,
    },
    #[error = "Index {index_id}, archive {archive_id} does not contain file {name}"]
    FileMissingName { index_id: u32, archive_id: u32, name: String },
    #[error = "Index {index_id} does not contain archive {archive_id}"]
    ArchiveMissing { index_id: u32, archive_id: u32 },
    #[error = "Index {index_id}, archive {archive_id} does not contain file {file} "]
    FileMissing { index_id: u32, archive_id: u32, file: u32 },
    #[error = "Index {index_id} does not contain archive {name}"]
    MissingName { index_id: u32, name: String },
    #[error = "Index {metadata.index_id} Archive {metadata.archive_id}: Crc does not match: {crc} !=  {metadata.crc}"]
    Crc { crc: i64, metadata: Metadata },
    #[error = "Index {metadata.index_id} Archive {metadata.archive_id}: Version does not match: {version} !=  {{v2}}"]
    Version { version: i64, metadata: Metadata },
    #[error = "Index {metadata.index_id}'s archive {metadata.archive_id} is blank"]
    Blank { metadata: Metadata },
    #[error = "Error retrieving {metadata}"]
    Corrupted { metadata: Metadata },
    #[error = "todo"]
    Other {},
}
*/
