//! Lantern Chat Client SDK

//#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::perf, clippy::must_use_candidate, clippy::complexity, clippy::suspicious)]
#![allow(clippy::bad_bit_mask, non_local_definitions)]

extern crate alloc;

#[cfg(all(feature = "typed-builder", feature = "bon"))]
compile_error!("'typed-builder' and 'bon' features are mutually exclusive");

#[macro_use]
extern crate serde;

#[cfg(feature = "borsh")]
#[macro_use]
extern crate borsh;

#[macro_use]
extern crate bitflags_serde_shim;

pub use models::{FxRandomState2, Snowflake};

#[macro_use]
pub mod models;

#[cfg(feature = "api")]
pub mod api;

#[cfg(feature = "driver")]
pub mod driver;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "gateway")]
pub mod gateway;

#[cfg(feature = "framework")]
pub mod framework;

#[cfg(feature = "framework_utils")]
pub mod framework_utils;
