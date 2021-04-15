![Build](https://github.com/mejrs/rs3-cache/workflows/Build/badge.svg)
![Tests](https://github.com/mejrs/rs3-cache/workflows/Tests/badge.svg)
[![Python](https://github.com/mejrs/rs3-cache/workflows/Python/badge.svg)](https://mejrs.github.io/doc/rs3cache/ffi/python/index.html "Python instructions")
[![Docs](https://github.com/mejrs/rs3-cache/workflows/Docs/badge.svg)](https://mejrs.github.io/doc/rs3cache/index.html "Documentation")

# RS3 cache tool

Tools and api for reading and interpreting the [RuneScape 3](https://www.runescape.com/community "RuneScape") game cache.

## Setup

- `git clone https://github.com/mejrs/rs3cache`.
- [Install the Rust compiler](https://doc.rust-lang.org/stable/book/ch01-01-installation.html "Installation - The Rust Programming Language").
- Configure rustup to use the nightly version: `rustup default nightly`.
- Navigate to this repository
- Compile the executable with `cargo build --release`.
- Either:
    - Create a system variable named `RUNESCAPE_CACHE_FOLDER` and set its value to where your cache is located.
    Typically, this is `%ProgramData%\Jagex\RuneScape`.
    - Copy the entire cache and place it in the `raw` folder.

* Optionally, build documentation with `cargo doc --no-deps --open`.

## Usage (executable)
- `target/release/rs3cache.exe --dump all`: save various archives as JSON in the `out` folder.
    - Use `target/release/rs3cache.exe --dump <archive>` to only dump a specific archive.
- `target/release/rs3cache.exe --render map`: render images of the game surface.
This exports them as small tiles, formatted as `<layer>/<zoom>/<plane>_<x>_<y>.png`, suitable for use with interactive map libraries such as [Leaflet](https://leafletjs.com/ "Leaflet - a JavaScript library for interactive maps"), as seen on [mejrs.github.io](https://mejrs.github.io/ "Mej's Map").
- `target/release/rs3cache.exe --assert cache_coherence`: checks whether the cache is in a coherent state.
- `target/release/rs3cache.exe --help` to see a list of commands

## Building Python wheels

The following instructions are only tested on **windows**. If you run into issues on other platforms, try following [here](https://github.com/PyO3/setuptools-rust#binary-wheels-on-linux "setuptools-rust") or build with [maturin](https://pypi.org/project/maturin/ "maturin") instead.

- `git clone https://github.com/mejrs/rs3cache`.
- Install [Python](https://www.python.org/downloads/ "Download Python"), version 3.6 or newer.
    - Check that pip is installed (`python -m pip --version`).
    - Install setuptools: `pip install setuptools`.
    - Install setuptools-rust: `pip install setuptools-rust`.
- [Install the Rust compiler](https://doc.rust-lang.org/stable/book/ch01-01-installation.html "Installation - The Rust Programming Language").
- Configure rustup to use the nightly version: `rustup default nightly`.
- Navigate to this repository and run `python setup.py install`.
- Either:
    - Create a system variable named `RUNESCAPE_CACHE_FOLDER` and set its value to where your cache is located.
      Typically, this is `%ProgramData%\Jagex\RuneScape`.
    - Copy the entire cache and place it in the `raw` folder.
- For examples, see `rs3cache/examples.py`.