use std::{
    collections::BTreeSet,
    sync::Mutex,
    time::{Duration, Instant},
};

use clap::{Parser, ValueEnum};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::cli::{Config, Dump};

/// Entry point for the program. Run the executable with `--help` for a list of commands.
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse();

    let start = Instant::now();

    #[cfg(all(feature = "rs3", not(feature = "mockdata"), not(feature = "save_mockdata")))]
    if config.assert_coherence {
        rs3cache_backend::index::assert_coherence(config.input.clone())?;
    }

    {
        let mut to_dump = config.dump.clone();

        if config.dump.contains(&Dump::Configs) {
            to_dump.extend_from_slice(Dump::configs())
        }

        if config.dump.contains(&Dump::All) {
            to_dump.extend_from_slice(Dump::value_variants())
        }

        let dump_sprites = to_dump.contains(&Dump::Sprites);

        #[cfg(feature = "rs3")]
        let dump_music = to_dump.contains(&Dump::Music);

        #[cfg(feature = "rs3")]
        let has_bars = [Dump::All, Dump::Configs, Dump::Sprites, Dump::Music];

        #[cfg(feature = "osrs")]
        let has_bars = [Dump::All, Dump::Configs, Dump::Sprites];

        #[cfg(feature = "legacy")]
        let has_bars = [];

        to_dump.retain(|item| !has_bars.contains(item));

        // Remove duplicates.
        let to_dump = to_dump.into_iter().collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();

        let progress = ProgressBar::new(to_dump.len() as u64).with_style(
            ProgressStyle::with_template(&format!(
                "   {} [{{bar:30}}] {{pos}}/{{len}}: {{wide_msg}}",
                style("Dumping").cyan().bright()
            ))
            .unwrap()
            .progress_chars("=> "),
        );

        let messages = Mutex::new(Vec::new());

        to_dump
            .into_par_iter()
            .map(|a| {
                let name = a.as_str();
                {
                    let mut messages = messages.lock().unwrap();
                    messages.push(name);
                    let msg = messages.join(", ");

                    progress.set_message(msg);

                    drop(messages);
                };

                let ret = a.call()(&config);
                progress.inc(1);
                {
                    let mut messages = messages.lock().unwrap();

                    messages.retain(|&n| n != name);
                    let msg = messages.join(", ");
                    progress.set_message(msg);

                    let news = if ret.is_err() {
                        format!("    {} failed dumping {a}", style("Error").red())
                    } else {
                        format!("    {} {name}", style("Dumped").green().bright())
                    };
                    progress.println(news);
                    drop(messages);
                };

                ret
            })
            .collect::<Result<Vec<_>, _>>()?;
        progress.finish_and_clear();

        if dump_sprites {
            Dump::Sprites.call()(&config)?;
        }

        #[cfg(feature = "rs3")]
        if dump_music {
            Dump::Music.call()(&config)?;
        }

        #[cfg(not(target_arch = "wasm32"))]
        for map in &(config.render) {
            map.call(&config)?;
        }
    }

    let dt = start.elapsed();

    if dt > Duration::from_secs(1) {
        println!("\nFinished program in {} s.", dt.as_secs());
    } else {
        println!("\nFinished program in {} ms.", dt.as_millis());
    }

    Ok(())
}
