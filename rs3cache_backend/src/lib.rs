//! Core cache interpreting utilities.

#![feature(
    lint_reasons,
    provide_any,
    error_generic_member_access,
    error_iter,
    cfg_eval,
    try_blocks,
    array_chunks,
    if_let_guard,
    let_chains,
    error_in_core
)]
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
    clippy::uninlined_format_args,
)]
#![deny(keyword_idents, macro_use_extern_crate)]
#![cfg(any(feature = "sqlite", feature = "dat2", feature = "dat"))]

use rs3cache_utils as utils;

pub mod arc;
pub mod buf;
pub mod decoder;
pub mod error;
pub mod hash;
pub mod index;
pub mod meta;
#[cfg(feature = "dat2")]
pub mod xtea;
