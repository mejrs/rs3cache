//! Core cache interpreting utilities.

#![feature(backtrace, error_iter, cfg_eval)]
#![allow(non_snake_case, unused_imports)]
#![warn(
     unused_qualifications,
    unused_import_braces,
    unused_extern_crates,
    rustdoc::broken_intra_doc_links,
    //missing_docs,
    rustdoc::missing_crate_level_docs,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub
)]
#![deny(keyword_idents, macro_use_extern_crate, non_ascii_idents)]
#![cfg(any(feature = "rs3", feature = "osrs", feature = "legacy"))]

use rs3cache_utils as utils;

pub mod arc;
pub mod buf;
pub mod decoder;
pub mod error;
pub mod hash;
pub mod index;
pub mod indextype;
pub mod meta;
#[cfg(feature = "osrs")]
pub mod xtea;
