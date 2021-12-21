mod absint;
mod builder;
mod diagnostics;
mod display;
mod graph;
mod ir;
mod key;
mod pass_manager;
mod region;
mod ssacfg;

// Public API.
pub use self::{
    absint::{LatticeSemantics, TypeKey},
    builder::OperationBuilder,
    diagnostics::{diagnostics_setup, LocationInfo},
    graph::Graph,
    ir::{
        Attribute, AttributeValue, BasicBlock, Intrinsic, IntrinsicTrait, Operation,
        SupportsVerification, Var,
    },
    key::Key,
    pass_manager::{
        AnalysisKey, AnalysisManager, OperationPass, OperationPassManager, PassManager,
    },
    region::Region,
    ssacfg::SSACFG,
};
