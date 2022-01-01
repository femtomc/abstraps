//! Target-specific code generation capabilities.
//!
//! The functionality in this module is mostly dialect-agnostic,
//! exposing common interfaces which may be used
//! to perform code generation.
//!
//! In general, when a code generation target provides an IR,
//! it is best to model the IR as a dialect, and then implement
//! code generation for that dialect (and express lowering
//! from other dialects as a conversion pass).
//!
//! Implementation of code generation has been organized through a common set of traits
//! (which capture traversal and emission on SSA-based block IRs).

#[cfg(feature = "clift")]
pub mod cranelift;

#[cfg(feature = "mlir")]
pub mod mlir;
