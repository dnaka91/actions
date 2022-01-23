//! Common utilities for GitHub Actions written in Rust.

#![forbid(unsafe_code)]
#![deny(missing_docs, rust_2018_idioms, clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

pub mod cli;
pub mod glob;
pub mod http;
pub mod tracing;
