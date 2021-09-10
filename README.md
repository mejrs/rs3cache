[![actions status](https://github.com/mejrs/rs3cache/workflows/CI/badge.svg)](https://github.com/mejrs/rs3cache/actions)

# RS3 cache tool

Tools and api for reading and interpreting the [RuneScape 3](https://www.runescape.com/community "RuneScape") game cache.

## Installing the command line tool.

- [Install the Rust compiler](https://doc.rust-lang.org/stable/book/ch01-01-installation.html "Installation - The Rust Programming Language").
- Install the tool with ```cargo +nightly install --git https://github.com/mejrs/rs3cache/ --features=rs3```.


## Usage 
- `rs3cache --help` to see a list of commands:

```text
USAGE:
    rs3cache.exe [FLAGS] [OPTIONS]

FLAGS:
        --assert-coherence
            Checks whether the cache is in a consistent state. Indices 14, 40, 54, 55 are not necessarily complete

    -h, --help
            Prints help information

    -V, --version
            Prints version information


OPTIONS:
        --dump <dump>...
            Allowed values: [all, sprites, locations, location_configs, location_configs_each, npc_configs, item_configs,
            maplabels, worldmaps, varbit_configs, structs, enums, underlays, overlays]

            Dumps the given archives.
        --input <input>
            The path where to look for the current cache. If omitted this falls back to the "RS3_CACHE_INPUT_FOLDER"
            environment variable and then to the current folder if not set [env: RS3_CACHE_INPUT_FOLDER=]  [default: ]
        --output <output>
            The path where to place output. If omitted this falls back to the "RS3_CACHE_OUTPUT_FOLDER" environment
            variable and then to the current folder if not set [env: RS3_CACHE_OUTPUT_FOLDER=]  [default: ]
        --render <render>...
            Allowed values: [all, map]

            This exports them as small tiles, formatted as `<layer>/<mapid>/<zoom>/<plane>_<x>_<y>.png`, suitable for
            use with interactive map libraries such as https://leafletjs.com/, as seen on https://mejrs.github.io/
```

## Building as a Python library.

### Using `setuptools-rust`

The following instructions are only tested on **windows**. If you run into issues on other platforms, try following [here](https://github.com/PyO3/setuptools-rust#binary-wheels-on-linux "setuptools-rust") or build with [maturin](https://pypi.org/project/maturin/ "maturin") instead.

- `git clone https://github.com/mejrs/rs3cache`.
- Install [Python](https://www.python.org/downloads/ "Download Python"), version 3.9 (lower versions may work).
    - Check that pip is installed (`python -m pip --version`).
    - Install setuptools: `pip install setuptools`.
    - Install setuptools-rust: `pip install setuptools-rust`.
- [Install the Rust compiler](https://doc.rust-lang.org/stable/book/ch01-01-installation.html "Installation - The Rust Programming Language").
- Navigate to this repository and run `python setup.py install`.
- Either:

### Using `maturin`

- `git clone https://github.com/mejrs/rs3cache`.
- Install [Python](https://www.python.org/downloads/ "Download Python"), version 3.9 (lower versions may work).
    - Check that pip is installed (`python -m pip --version`).
    - Install maturin: `pip install maturin`.
- [Install the Rust compiler](https://doc.rust-lang.org/stable/book/ch01-01-installation.html "Installation - The Rust Programming Language").
- Configure rustup to use the nightly version: `rustup default nightly`.
- Navigate to this repository and build a Python wheel with `maturin build --cargo-extra-args="--features=rs3,pyo3"`.


- For examples, see `rs3cache/examples.py`.