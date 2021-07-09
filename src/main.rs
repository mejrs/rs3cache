use std::time::Instant;

use clap::{load_yaml, App};
#[allow(unused_imports)]
use rs3cache::{cache::index, definitions, renderers::map};

#[cfg(feature = "rs3")]
const BASE: &str = "out/rs3";

#[cfg(feature = "osrs")]
const BASE: &str = "out/osrs";

/// Entry point for the program. Run the executable with `--help` for a list of commands.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let start = Instant::now();

    let archives = matches.values_of("archives").unwrap_or_default();

    for archive in archives {
        match archive {
            "location_configs" => definitions::location_configs::export(format!("{}/location_configs", BASE))?,
            "location_configs_each" =>  definitions::location_configs::export_each(format!("{}/location_configs", BASE))?,
            "locations" => definitions::locations::export()?,
            "npc_configs" => definitions::npc_configs::export()?,
            "item_configs" => definitions::item_configs::export()?,
            "maplabels" => definitions::maplabel_configs::export()?,
            "overlays" => definitions::overlays::export()?,
            "sprites" => definitions::sprites::save_all()?,
            #[cfg(feature = "osrs")]
            "textures" => definitions::textures::export(format!("{}/textures", BASE))?,
            "worldmaps" => {
                definitions::worldmaps::dump_big()?;
                definitions::worldmaps::dump_small()?;
                definitions::worldmaps::export_pastes()?;
                definitions::worldmaps::export_zones()?;
            }
            "varbit_configs" => definitions::varbit_configs::export()?,
            "structs" => definitions::structs::export()?,
            "enums" => definitions::enums::export()?,

            "all" => {
                definitions::locations::export()?;
                definitions::npc_configs::export()?;
                definitions::item_configs::export()?;
                definitions::maplabel_configs::export()?;
                definitions::overlays::export()?;
                {
                    definitions::worldmaps::dump_big()?;
                    definitions::worldmaps::dump_small()?;
                    definitions::worldmaps::export_pastes()?;
                    definitions::worldmaps::export_zones()?;
                }
                definitions::varbit_configs::export()?;
                definitions::structs::export()?;
                definitions::enums::export()?;
                #[cfg(feature = "osrs")]
                definitions::textures::export(format!("{}/textures", BASE))?;

                {
                    definitions::location_configs::export(format!("{}/location_configs", BASE))?;
                    definitions::location_configs::export_each(format!("{}/location_configs", BASE))?;
                }
                definitions::sprites::save_all()?;
            }
            _ => unreachable!(),
        }
    }

    if matches.is_present("layer") {
        match matches.value_of("layer") {
            Some("all") => {
                map::render(format!("{}/map_squares", BASE))?;
            }
            Some("map") => {
                map::render(format!("{}/map_squares", BASE))?;
            }
            _ => unreachable!(),
        }
    };

    if matches.is_present("assertion") {
        #[cfg(all(feature = "rs3", not(any(feature = "mockdata", feature = "save_mockdata"))))]
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
