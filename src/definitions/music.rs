use std::{
    fs::{self, File},
    io::Write,
    process::Command,
};

use fstrings::{f, format_args_f};
use path_macro::path;

use crate::{
    cache::{buf::Buffer, error::CacheResult, index::CacheIndex, indextype::IndexType},
    definitions::enums::{Enum, Value},
};

pub fn export_each(config: &crate::cli::Config) -> CacheResult<()> {
    let enum_archives = CacheIndex::new(IndexType::ENUM_CONFIG, &config.input)?;
    let mut archive = enum_archives.archive(5)?;
    let music_names = Enum::deserialize(5 << 8 | 65, archive.take_file(&65)?);
    let music_indices = Enum::deserialize(5 << 8 | 71, archive.take_file(&71)?);
    let audio_archives = CacheIndex::new(IndexType::AUDIOSTREAMS, &config.input)?;

    for (archive_id, name) in music_names.variants.into_iter() {
        let name = match name {
            Value::String(s) if s.chars().all(|c| c == ' ') =>  f!("Unnamed track {archive_id}"),
            // Check for bad filenames
            // Almost never happens, so we check first before possibly creating a new string
            Value::String(s) if s.chars().any(|c| ['?','/', '\\'].contains(&c)) =>  s.chars().filter(|c| ['?','/', '\\'].contains(c)).collect(),
            Value::String(s) => s,
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
            Ok(mut file) => file.take_file(&0).unwrap(),
            _ => {
                // Seems like things are lazily loaded.
                println!("Unable to create \"{}\".", name);
                continue;
            }
        };

        let (jaga, data) = decode_first(first);

        let out = path_macro::path!(config.output / "music" / name);
        fs::create_dir_all(&out).unwrap();

        let file_name = path!(&out / f!("{music_archive_id}.ogg"));
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
            let more_data = audio_archives.archive(archive_id).unwrap().take_file(&0).unwrap();
            let file_name = path!(&out / f!("{archive_id}.ogg"));
            let mut file = File::create(&file_name).unwrap();
            file.write_all(&more_data).unwrap();
            command.arg(file_name);
        }
        
        command.arg(path!(config.output / "music" / f!("{name}.ogg")));
        command.output().unwrap();
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct Jaga {
    int_1: u32,
    int_2: u32,
    sample_frequency: u32,
    int_3: u32,
    chunks: Vec<ChunkDescriptor>,
}

#[derive(Debug, Clone)]
pub struct ChunkDescriptor {
    position: u32,
    length: u32,
    archive_id: u32,
}

fn decode_first(data: Vec<u8>) -> (Jaga, Vec<u8>) {
    let mut buf = Buffer::new(data);
    let signature = buf.read_array::<4>();
    assert_eq!(&signature, b"JAGA");

    let int_1 = buf.read_unsigned_int();
    let int_2 = buf.read_unsigned_int();
    let sample_frequency = buf.read_unsigned_int();
    let int_3 = buf.read_unsigned_int();
    let chunk_count = buf.read_unsigned_int();
    let chunks = (0..chunk_count)
        .map(|position| ChunkDescriptor {
            position,
            length: buf.read_unsigned_int(),
            archive_id: buf.read_unsigned_int(),
        })
        .collect();
    (
        Jaga {
            int_1,
            int_2,
            sample_frequency,
            int_3,
            chunks,
        },
        buf.remainder(),
    )
}
