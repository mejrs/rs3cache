use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    env::{self, VarError},
    fs::{self, File},
    io::{self, Cursor, Read, Seek, SeekFrom, Write},
    marker::PhantomData,
    ops::RangeInclusive,
    path::{Path, PathBuf},
};

use bytes::{Buf, Bytes};
use fstrings::{f, format_args_f};
use itertools::iproduct;
use path_macro::path;

use crate::{
    arc::Archive,
    buf::BufExtra,
    decoder,
    error::{CacheError, CacheResult},
    index::{CacheIndex, IndexState, Initial},
    indextype::IndexType,
    meta::{IndexMetadata, Metadata},
};

impl<S> CacheIndex<S>
where
    S: IndexState,
{
    fn get_entry(a: u32, b: u32, folder: impl AsRef<Path>) -> CacheResult<(u32, u32)> {
        let file = path!(folder / "cache" / f!("main_file_cache.idx{a}"));
        let entry_data = fs::read(&file).map_err(|e| CacheError::CacheNotFoundError(e, file))?;
        let mut buf = Cursor::new(entry_data);
        buf.seek(SeekFrom::Start((b * 6) as _)).unwrap();
        Ok((buf.try_get_uint(3)? as u32, buf.try_get_uint(3)? as u32))
    }

    fn read_index(&self, a: u32, b: u32) -> CacheResult<Vec<u8>> {
        let mut buffer = Cursor::new(&self.file);

        let (length, mut sector) = Self::get_entry(a, b, &self.path)?;

        let mut read_count = 0;
        let mut part = 0;
        let mut data = Vec::with_capacity(length as _);

        while sector != 0 {
            buffer.seek(SeekFrom::Start((sector * 520) as _))?;
            let (_header_size, current_archive, block_size) = if b >= 0xFFFF {
                (10, buffer.get_i32(), 510.min(length - read_count))
            } else {
                (8, buffer.get_u16() as _, 512.min(length - read_count))
            };

            let current_part = buffer.get_u16();
            let new_sector = buffer.get_uint(3) as u32;
            let _current_index = buffer.get_u8();

            assert_eq!(b, current_archive as u32);
            assert_eq!(part, current_part as u32);

            part += 1;
            read_count += block_size;
            sector = new_sector;

            data.extend(buffer.copy_to_bytes(block_size as _));
        }
        Ok(data)
    }

    pub fn get_file(&self, metadata: &Metadata) -> CacheResult<Bytes> {
        let data = self.read_index(metadata.index_id(), metadata.archive_id())?;
        if metadata.index_id() == 0 {
            // The caller of this function is responsible for unpacking the .jag format
            return Ok(Bytes::from(data));
        }
        Ok(decoder::decompress(data, None)?)
    }

    pub fn archive_by_name(&self, name: String) -> CacheResult<Bytes> {
        let hash = crate::hash::hash_archive(&name);
        for (_, m) in self.metadatas.iter() {
            if m.name() == Some(hash) {
                return self.get_file(m);
            }
        }
        Err(CacheError::ArchiveNotFoundError(0, 0))
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
    pub fn new(index_id: u32, folder: impl AsRef<Path>) -> CacheResult<CacheIndex<Initial>> {
        let file = path!(folder / "cache/main_file_cache.dat");

        let file = fs::read(&file).map_err(|e| CacheError::CacheNotFoundError(e, file))?.into_boxed_slice();

        Ok(Self {
            path: folder.as_ref().to_path_buf(),
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
