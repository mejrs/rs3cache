use std::time::Instant;

use rs3cache::cli::Config;
use structopt::StructOpt;

/// Entry point for the program. Run the executable with `--help` for a list of commands.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_args();

    let start = Instant::now();

    #[cfg(all(feature = "rs3", not(feature = "mockdata")))]
    if config.assert_coherence {
        rs3cache::cache::index::assert_coherence(&config.input)?;
    }

    for archive in &(config.dump) {
        archive.call(&config)?;
    }

    for map in &(config.render) {
        map.call(&config)?;
    }

    println!("\nFinished program in {} s.", start.elapsed().as_secs());

    Ok(())
}
