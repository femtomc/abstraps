//! Core functionality, including IR ([`Operation`]) definition,
//! pass manager implementation, and declarative macro functionality
//! for intrinsic/operation extension.

mod absint;
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
    absint::{Interpreter, LatticeJoin, LatticeSemantics, Signature},
    builder::OperationBuilder,
    diagnostics::{diagnostics_color_disable, diagnostics_setup, LocationInfo},
    interfaces::*,
    ir::{
        Attribute, AttributeValue, BasicBlock, Intrinsic, Operation, SupportsInterfaceTraits, Var,
    },
    pass_manager::{
        AnalysisKey, AnalysisManager, AnalysisPass, OperationPass, OperationPassManager,
        PassManager,
    },
    region::{Graph, Region, SSACFG},
};
