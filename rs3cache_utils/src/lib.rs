//! Miscellanious utilities.
#![forbid(unsafe_code)]
#![warn(
    unused_crate_dependencies,
    unused_imports,
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
#![deny(keyword_idents, macro_use_extern_crate)]

/// Adapters for iterators.
pub mod adapters;

/// Progress bars
pub mod bar;

/// Various colour constants.
pub mod color;

/// Lazy primitives.
pub mod lazy;

/// Clamps a [`Range`](std::ops::Range) to a certain interval.
pub mod rangeclamp;

pub mod slice;
