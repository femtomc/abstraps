mod attributes;
mod intrinsics;
mod passes;
mod traits;

pub use self::{
    attributes::{Symbol, SymbolTable},
    intrinsics::{Func, Module},
    passes::PopulateSymbolTablePass,
    traits::{ProvidesSymbol, ProvidesSymbolTable, RequiresTerminators, Terminator},
};
