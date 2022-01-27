#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, clippy::all, clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions
)]

pub mod apt;
pub mod archive;
pub mod cargo;
pub mod rustup;
pub mod toolchain;
pub mod triple;
