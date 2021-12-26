//! This dialect supports operations and lattices which express
//! meaningful semantics for symbolic execution.
//!
//! The design and implementation follows an abstract interpretation
//! approach, based on [Rosette](https://emina.github.io/rosette/),
//! but applied to the SSA-based (extensible concepts) of MLIR.
//!
//! **This dialect is experimental.**
