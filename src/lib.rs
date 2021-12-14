/*

   This file is part of `abstraps`. License is MIT.

*/

#![doc = include_str!("../README.md")]
#![allow(dead_code)]
#![warn(missing_docs)]

pub mod backends;
pub mod dialects;
//pub mod interp;
pub mod core;

#[macro_use]
extern crate alloc;
