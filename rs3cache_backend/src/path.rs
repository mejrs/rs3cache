use core::fmt;
use std::{ffi::OsStr, path::Path, sync::Arc};

use clap::{parser::ValueSource, ArgMatches, Command, FromArgMatches};

pub const INPUT: &str = if cfg!(feature = "sqlite") {
    "RS3_CACHE_INPUT_FOLDER"
} else if cfg!(feature = "dat2") {
    "OSRS_CACHE_INPUT_FOLDER"
} else if cfg!(feature = "dat") {
    "LEGACY_CACHE_INPUT_FOLDER"
} else {
    unimplemented!()
};

use std::ffi::OsString;
#[derive(Clone, Debug, Default)]
pub enum CachePath {
    #[default]
    Default,
    Env(Arc<Path>),
    CommandLine(Arc<Path>),
    Argument(Arc<Path>),
}

impl fmt::Display for CachePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.as_ref();

        #[cfg(not(target_arch = "wasm32"))]
        let path = ::path_absolutize::Absolutize::absolutize(path).unwrap_or(std::borrow::Cow::Borrowed(path));

        fmt::Display::fmt(&path.display(), f)
    }
}

impl AsRef<Path> for CachePath {
    fn as_ref(&self) -> &Path {
        match self {
            CachePath::Default => Path::new(""),
            CachePath::Env(p) | CachePath::CommandLine(p) | CachePath::Argument(p) => p,
        }
    }
}

impl FromArgMatches for CachePath {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        let default = OsString::default();

        let path: &OsString = matches.try_get_one("input").unwrap().unwrap_or(&default);
        let path: Arc<Path> = Path::new(path).into();

        let ret = match matches.value_source("input") {
            Some(ValueSource::EnvVariable) => CachePath::Env(path),
            Some(ValueSource::CommandLine) => CachePath::CommandLine(path),
            _ => CachePath::Default,
        };

        Ok(ret)
    }
    fn update_from_arg_matches(&mut self, _matches: &ArgMatches) -> Result<(), clap::Error> {
        Ok(())
    }
}

impl clap::Args for CachePath {
    fn augment_args(cmd: Command) -> Command {
        let arg = clap::Arg::new("input")
            .value_name("INPUT")
            .help("The path where to look for the current cache")
            .long("input")
            .env(INPUT)
            .default_value(OsStr::new("..."))
            .value_parser(clap::builder::OsStringValueParser::new())
            .required(false);

        cmd.arg(arg)
    }
    fn augment_args_for_update(cmd: Command) -> Command {
        let arg = clap::Arg::new("input")
            .value_name("INPUT")
            .help("The path where to look for the current cache")
            .long("input")
            .env(INPUT)
            .default_value(OsStr::new("..."))
            .value_parser(clap::builder::OsStringValueParser::new())
            .required(false);
        cmd.arg(arg)
    }
}

pub struct LocationHelp<'p>(pub &'p CachePath);

impl fmt::Display for LocationHelp<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            CachePath::CommandLine(path) => writeln!(f, "looking in this directory because `--input {path:?}` was given on the command line")?,
            CachePath::Argument(path) => writeln!(
                f,
                "looking in this directory because the path {path:?} was given as as a function argument"
            )?,
            CachePath::Env(path) => writeln!(
                f,
                "looking in this directory because the path {path:?} was retrieved from the `{INPUT}` environment variable"
            )?,
            CachePath::Default => writeln!(f, "looking in the current directory because no path was given")?,
        }

        Ok(())
    }
}
