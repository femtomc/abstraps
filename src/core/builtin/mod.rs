//! This dialect supports primitive operations, traits, attributes,
//! and passes which are fundamental to usage of the framework.
//!
//! The implementation follows the [MLIR
//! implementation](https://mlir.llvm.org/docs/Dialects/Builtin/)
//! as closely as possible.

mod attributes;
mod primitives;
mod traits;

pub use self::{
    attributes::{
        ConstantAttr, LinkageAttr, ProvidesConstantAttr, ProvidesLinkageAttr, ProvidesSymbolAttr,
        ProvidesSymbolTableAttr, SymbolAttr, SymbolTableAttr,
    },
    primitives::{Func, Module},
    traits::{FunctionLike, NonVariadic, RequiresTerminators, Terminator},
};
