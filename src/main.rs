#[allow(unused_imports)]
use rs3cache::{cache::index, definitions, renderers::map};

use clap::{load_yaml, App};

use std::time::Instant;

/// Entry point for the program. Run the executable with `--help` for a list of commands.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let start = Instant::now();

    let archives = matches.values_of("archives").unwrap_or_default();

    for archive in archives {
        match archive {
            "location_configs" => definitions::location_configs::export()?,
            "location_configs_each" => definitions::location_configs::export_each()?,
            "locations" => definitions::locations::export()?,
            "npc_configs" => definitions::npc_configs::export()?,
            "maplabels" => definitions::maplabel_configs::export()?,
            "sprites" => definitions::sprites::save_all()?,
            "worldmaps" => {
                definitions::worldmaps::dump_big()?;
                definitions::worldmaps::dump_small()?;
                definitions::worldmaps::export_pastes()?;
                definitions::worldmaps::export_zones()?;
            }

            "all" => {
                {
                    definitions::location_configs::export()?;
                    definitions::location_configs::export_each()?;
                }
                definitions::locations::export()?;
                definitions::npc_configs::export()?;
                definitions::maplabel_configs::export()?;
                definitions::sprites::save_all()?;
                {
                    definitions::worldmaps::dump_big()?;
                    definitions::worldmaps::dump_small()?;
                    definitions::worldmaps::export_pastes()?;
                    definitions::worldmaps::export_zones()?;
                }
            }
            _ => unreachable!(),
        }
    }

    if matches.is_present("layer") {
        match matches.value_of("layer") {
            Some("all") => {
                map::render()?;
            }
            Some("map") => {
                map::render()?;
            }
            _ => unreachable!(),
        }
    };

    if matches.is_present("assertion") {
        #[cfg(not(any(feature = "mockdata", feature = "save_mockdata")))]
        match matches.value_of("assertion") {
            Some("cache_coherence") => {
                index::assert_coherence()?;
            }
            _ => unreachable!(),
        }
    }

    println!("\nFinished program in {} s.", start.elapsed().as_secs());
    Ok(())
}
