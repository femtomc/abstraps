//! Target-specific code generation capabilities.
//!
//! Code generation has been organized through a common set of traits
//! (which capture traversal and emission on SSA-based block IRs).

#[cfg(feature = "mlir")]
pub mod mlir;

#[cfg(feature = "clift")]
pub mod cranelift;
