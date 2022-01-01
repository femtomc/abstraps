//! This dialect supports primitive operations, traits, attributes,
//! and passes which are fundamental to usage of the framework.
//!
//! The implementation follows the [MLIR
//! implementation](https://mlir.llvm.org/docs/Dialects/Builtin/)
//! as closely as possible.

mod attributes;
mod intrinsics;
mod lattice;
mod passes;
mod traits;

pub use self::{
    attributes::{
        ConstantAttr, LinkageAttr, ProvidesConstantAttr, ProvidesLinkageAttr, ProvidesSymbolAttr,
        ProvidesSymbolTableAttr, SymbolAttr, SymbolTableAttr,
    },
    intrinsics::{Func, Module},
    lattice::BuiltinLattice,
    passes::PopulateSymbolTablePass,
    traits::{FunctionLike, NonVariadic, RequiresTerminators, Terminator},
};
