//! # Protomask library
//!
//! *Note: There is a fair chance you are looking for `src/cli/main.rs` instead of this file.*

#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

pub mod metrics;
pub mod nat;
mod packet;
mod profiling;
