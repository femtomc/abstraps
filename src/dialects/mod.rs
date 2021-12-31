//! IR intrinsic dialects which are supported internally by the framework.
//!
//! These dialects consist of common concepts found throughout
//! multiple other compilers/intermediate representations, e.g.
//! arithmetic over numeric types, memory allocation and referencing,

pub mod arith;
pub mod base;
pub mod builtin;
pub mod memref;
pub mod symbolic;
