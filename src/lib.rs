#![doc = include_str!("../README.md")]
#![warn(rustdoc::missing_doc_code_examples)]

pub mod core;
pub mod dialects;

extern crate color_eyre;
extern crate tracing;
extern crate tracing_subscriber;

#[macro_use]
extern crate lazy_static;

pub use color_eyre::{eyre::bail, Report};
