/*

   This file is part of `abstraps`. License is MIT.

*/

#![allow(dead_code)]
#![no_std]

#[macro_use]
extern crate alloc;

pub mod ir;
pub mod typeinf;

#[cfg(feature = "mlir")]
pub mod mlir;
