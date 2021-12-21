#![doc = include_str!("../README.md")]

//pub mod backends;
pub mod dialects;

mod core;
pub use self::core::*;

#[macro_use]
extern crate alloc;
extern crate color_eyre;
extern crate tracing;
extern crate tracing_subscriber;

#[macro_use]
extern crate lazy_static;
