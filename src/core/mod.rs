//mod absint;
mod builder;
mod diagnostics;
mod display;
#[macro_use]
mod interfaces;
mod ir;
mod pass_manager;
mod region;

// Public API.
pub use self::{
    //    absint::{LatticeJoin, LatticeSemantics, TypeKey},
    builder::OperationBuilder,
    diagnostics::{diagnostics_setup, LocationInfo},
    interfaces::*,
    ir::{
        Attribute, AttributeValue, BasicBlock, Intrinsic, Operation, SupportsInterfaceTraits, Var,
    },
    pass_manager::{
        AnalysisKey, AnalysisManager, OperationPass, OperationPassManager, PassManager,
    },
    region::{Graph, Region, SSACFG},
};
