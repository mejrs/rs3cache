use std::time::{Duration, Instant};

use rs3cache::cli::Config;
use structopt::StructOpt;

/// Entry point for the program. Run the executable with `--help` for a list of commands.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_args();

    let start = Instant::now();

    #[cfg(all(feature = "rs3", not(feature = "mockdata"), not(feature = "save_mockdata")))]
    if config.assert_coherence {
        rs3cache::cache::index::assert_coherence(&config.input)?;
    }

    for archive in &(config.dump) {
        archive.call(&config)?;
    }

    #[cfg(not(target_arch = "wasm32"))]
    for map in &(config.render) {
        map.call(&config)?;
    }

    let dt = start.elapsed();

    if dt > Duration::from_secs(1) {
        println!("\nFinished program in {} s.", dt.as_secs());
    } else {
        println!("\nFinished program in {} ms.", dt.as_millis());
    }

    Ok(())
}
