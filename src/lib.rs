/*

   This file is part of `abstraps`. License is MIT.

*/

#[doc = include_str!("../README.md")]
#![allow(dead_code)]
#![warn(missing_docs)]
#[macro_use]
extern crate alloc;

pub mod backends;
pub mod builder;
pub mod interp;
pub mod ir;
