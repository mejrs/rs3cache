[![actions status](https://github.com/mejrs/rs3cache/workflows/CI/badge.svg)](https://github.com/mejrs/rs3cache/actions)

# RS3 cache tool

Tools and api for reading and interpreting the [RuneScape 3](https://www.runescape.com/community "RuneScape") game cache.

## Installing the command line tool.

- [Install the Rust compiler](https://doc.rust-lang.org/stable/book/ch01-01-installation.html "Installation - The Rust Programming Language").
- Install the tool with 
   ```text
    cargo +nightly install --git https://github.com/mejrs/rs3cache/ rs3 --features=rs3
    ```

## Usage 
- `rs3 --help` to see a list of commands:

```text
USAGE:
    rs3.exe [OPTIONS]

OPTIONS:
        --assert-coherence      Checks whether the cache is in a consistent state. Indices 14, 40,
                                54, 55 are not necessarily complete
        --dump <DUMP>...        Dumps the given archives [possible values: all, configs, music,
                                achievements, sprites, locations, locations_each, tiles_each,
                                location_configs, location_configs_each, npc_config, item_configs,
                                maplabels, worldmaps, varbit_configs, structs, enums, underlays,
                                overlays]
    -h, --help                  Print help information
        --input <INPUT>         The path where to look for the current cache [env:
                                RS3_CACHE_INPUT_FOLDER=C:\ProgramData\Jagex\RuneScape] [default: ]
        --output <OUTPUT>       The path where to place output [env: RS3_CACHE_OUTPUT_FOLDER=]
                                [default: ]
        --render <RENDER>...    This exports them as small tiles, formatted as
                                `<layer>/<mapid>/<zoom>/<plane>_<x>_<y>.png`, suitable for use with
                                interactive map libraries such as <https://leafletjs.com/>, as seen
                                on <https://mejrs.github.io/> [possible values: all, map]
```

## Building as a Python library.

### Using `maturin`

- Clone the repository:
   ```text
   git clone https://github.com/mejrs/rs3cache
   ```
- Install [Python](https://www.python.org/downloads/ "Download Python"), version 3.7 or newer.
    - Check that pip is installed (`python -m pip --version`).
    - Install maturin: `pip install maturin`.
- [Install the Rust compiler](https://doc.rust-lang.org/stable/book/ch01-01-installation.html "Installation - The Rust Programming Language").
- Navigate to the `/rs3_py` folder and build a Python wheel with 
    ```text
    maturin build
    ```
- Using the wheel from above, run
    ```text
    pip install <path to wheel>
    ```

- For examples, see `rs3cache/examples`.

### Using `setuptools-rust`

The following instructions are only tested on **windows**. If you run into issues on other platforms, try following [here](https://github.com/PyO3/setuptools-rust#binary-wheels-on-linux "setuptools-rust") or build with [maturin](https://pypi.org/project/maturin/ "maturin") instead.

- Clone the repository:
   ```text
   git clone https://github.com/mejrs/rs3cache
   ```
- Install [Python](https://www.python.org/downloads/ "Download Python"), version 3.7 or newer..
    - Check that pip is installed (`python -m pip --version`).
    - Install setuptools: `pip install setuptools`.
    - Install setuptools-rust: `pip install setuptools-rust`.
- [Install the Rust compiler](https://doc.rust-lang.org/stable/book/ch01-01-installation.html "Installation - The Rust Programming Language").
- Navigate to the `/rs3_py` folder and build a Python wheel with 
    ```text
    python setup.py install
    ```

- For examples, see `rs3cache/examples`.