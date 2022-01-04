//! IR intrinsic dialects which are supported internally by the framework.
//!
//! These dialects consist of common concepts found throughout
//! multiple other compilers/intermediate representations, e.g.
//! arithmetic over numeric types, memory allocation and referencing.
//!
//! Some of these dialects are designed as lower-level targets
//! which support code generation via specific backends
//! (like [cranelift](https://github.com/bytecodealliance/wasmtime/tree/main/cranelift)). Dialects
//! can target other dialects which support code generation by utilizing
//! the pass framework to write a conversion pass.

#[cfg(feature = "arith")]
pub mod arith;

#[cfg(feature = "base")]
pub mod base;

#[cfg(feature = "builtin")]
pub mod builtin;

#[cfg(feature = "clift")]
pub mod cranelift;

#[cfg(feature = "memref")]
pub mod memref;

#[cfg(feature = "scf")]
pub mod scf;

#[cfg(feature = "symbolic")]
pub mod symbolic;
