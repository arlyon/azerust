//! game
//!
//! This crate models much of the core gameplay systems of
//! World of Warcraft.

#![forbid(unsafe_code)]
#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    clippy::useless_conversion,
    clippy::unwrap_used,
    clippy::todo,
    clippy::unimplemented
)]

pub mod accounts;
pub mod realms;
pub mod types;
