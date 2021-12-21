mod absint;
mod builder;
mod diagnostics;
mod display;
pub mod interfaces;
mod ir;
mod key;
mod pass_manager;
mod region;

// Public API.
pub use self::{
    absint::{LatticeJoin, LatticeSemantics, TypeKey},
    builder::OperationBuilder,
    diagnostics::{diagnostics_setup, LocationInfo},
    ir::{
        Attribute, AttributeValue, BasicBlock, Intrinsic, IntrinsicTrait, Operation,
        SupportsVerification, Var,
    },
    key::Key,
    pass_manager::{
        AnalysisKey, AnalysisManager, OperationPass, OperationPassManager, PassManager,
    },
    region::{Graph, Region, SSACFG},
};
