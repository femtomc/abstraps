/*

   This file is part of `abstraps`. License is MIT.

*/

#![allow(dead_code)]
#![warn(missing_docs)]
#[doc = include_str!("../README.md")]
#[macro_use]
extern crate alloc;

pub mod backends;
pub mod builder;
pub mod interp;
pub mod ir;
