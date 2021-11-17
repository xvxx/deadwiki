#![deny(
    anonymous_parameters,
    clippy::all,
    const_err,
    illegal_floating_point_literal_pattern,
    late_bound_lifetime_arguments,
    path_statements,
    patterns_in_fns_without_body,
    rust_2018_idioms,
    trivial_numeric_casts,
    unused_extern_crates
)]
#![warn(
    clippy::dbg_macro,
    clippy::decimal_literal_representation,
    clippy::get_unwrap,
    clippy::nursery,
    clippy::pedantic,
    clippy::todo,
    clippy::unimplemented,
    clippy::use_debug,
    clippy::all,
    unused_qualifications,
    variant_size_differences
)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::needless_doctest_main)] // main is useful for docs
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::shadow_unrelated)] // warnings annoyingly show up in macros
#![allow(clippy::module_name_repetitions)] // helps with context
#![allow(clippy::inline_always)] // let me inline hot functions
#![allow(clippy::unused_self)] // fixes clippy errors with different features
#![allow(missing_docs)]
#![allow(clippy::missing_errors_doc)]


#[macro_use]
pub mod shell;
pub mod app;
pub mod db;
mod hatter;
pub mod markdown;
mod page;
pub mod sync;
pub mod utils;

pub use {crate::hatter::Hatter, page::Page};
