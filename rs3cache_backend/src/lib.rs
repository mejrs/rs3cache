//! Core cache interpreting utilities.

#![feature(error_generic_member_access)]
#![feature(error_iter)]
#![feature(cfg_eval)]
#![feature(try_blocks)]
#![cfg_attr(feature = "dat2", feature(iter_array_chunks))]
#![allow(clippy::result_large_err, unexpected_cfgs)]
#![warn(
    unused_qualifications,
    unused_import_braces,
    unused_extern_crates,
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    trivial_casts,
    trivial_numeric_casts,
    clippy::uninlined_format_args
)]
#![deny(keyword_idents, macro_use_extern_crate)]
#![cfg(any(feature = "sqlite", feature = "dat2", feature = "dat"))]

pub mod arc;
pub mod buf;
pub mod decoder;
pub mod error;
pub mod hash;
pub mod index;
pub mod meta;
pub mod path;
#[cfg(feature = "dat2")]
pub mod xtea;
