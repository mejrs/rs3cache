use std::{
    fs::{self, File},
    io::Write,
    process::Command,
};

use bytes::{Buf, Bytes};
use path_macro::path;

use crate::{
    cache::{buf::BufExtra, error::CacheResult, index::CacheIndex},
    definitions::{
        enums::{Enum, Value},
        indextype::IndexType,
    },
};

pub fn export_each(config: &crate::cli::Config) -> CacheResult<()> {
    let enum_archives = CacheIndex::new(IndexType::ENUM_CONFIG, config.input.clone())?;
    let archive = enum_archives.archive(5)?;
    let music_names = Enum::deserialize(5 << 8 | 65, archive.file(&65)?);
    let music_indices = Enum::deserialize(5 << 8 | 71, archive.file(&71)?);
    let audio_archives = CacheIndex::new(IndexType::AUDIOSTREAMS, config.input.clone())?;

    for (archive_id, name) in music_names.variants.into_iter() {
        let name = match name {
            Value::String(s) if s.chars().all(|c| c == ' ') => format!("Unnamed track {archive_id}"),
            // Check for bad filenames
            // Almost never happens, so we check first before possibly creating a new string
            Value::String(s) if s.chars().any(|c| ['?', '/', '\\'].contains(&c)) => s.chars().filter(|c| ['?', '/', '\\'].contains(c)).collect(),
            Value::String(s) => (*s).to_owned(),
            _ => unreachable!(),
        };

        let music_archive_id = match music_indices.variants.get(&archive_id) {
            Some(Value::Integer(i)) => *i as u32,
            Some(_) => unreachable!(),
            None => {
                println!("Unable to create \"{}\".", name);
                continue;
            }
        };

        let first = match audio_archives.archive(music_archive_id) {
            Ok(file) => file.file(&0).unwrap(),
            _ => {
                // Seems like things are lazily loaded.
                println!("Unable to create \"{}\".", name);
                continue;
            }
        };

        let (jaga, data) = decode_first(first);

        let out = path_macro::path!(config.output / "music" / name);
        fs::create_dir_all(&out).unwrap();

        let file_name = path!(&out / format!("{music_archive_id}.ogg"));
        let mut file = File::create(&file_name).unwrap();
        file.write_all(&data).unwrap();

        // Prepare a process that invokes `Sox` to concatenate all these files.
        // This is really scuffed, but SoX seems to be only program capable of handling it
        // Simply concatenating the files should normally work for .ogg files, but not for these...
        let mut command = Command::new("sox");
        command.arg(file_name);

        // The first one is the one that's already written
        for chunk in jaga.chunks.iter().skip(1) {
            let archive_id = chunk.archive_id;
            let more_data = audio_archives.archive(archive_id).unwrap().file(&0).unwrap();
            let file_name = path!(&out / format!("{archive_id}.ogg"));
            let mut file = File::create(&file_name).unwrap();
            file.write_all(&more_data).unwrap();
            command.arg(file_name);
        }

        command.arg(path!(config.output / "music" / format!("{name}.ogg")));
        command.output().unwrap();
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct Jaga {
    _int_1: u32,
    _int_2: u32,
    _sample_frequency: u32,
    _int_3: u32,
    chunks: Vec<ChunkDescriptor>,
}

#[derive(Debug, Clone)]
pub struct ChunkDescriptor {
    _position: u32,
    _length: u32,
    archive_id: u32,
}

fn decode_first(mut buffer: Bytes) -> (Jaga, Bytes) {
    let signature = buffer.get_array::<4>();
    assert_eq!(&signature, b"JAGA");

    let _int_1 = buffer.get_u32();
    let _int_2 = buffer.get_u32();
    let _sample_frequency = buffer.get_u32();
    let _int_3 = buffer.get_u32();
    let chunk_count = buffer.get_u32();
    let chunks = (0..chunk_count)
        .map(|_position| ChunkDescriptor {
            _position,
            _length: buffer.get_u32(),
            archive_id: buffer.get_u32(),
        })
        .collect();
    (
        Jaga {
            _int_1,
            _int_2,
            _sample_frequency,
            _int_3,
            chunks,
        },
        buffer,
    )
}
