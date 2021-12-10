/*

   This file is part of `abstraps`. License is MIT.

*/

#[doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![allow(dead_code)]

#[macro_use]
extern crate alloc;

pub mod backends;
pub mod builder;
pub mod interp;
pub mod ir;
