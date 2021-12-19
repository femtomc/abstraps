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
    builder::OperationBuilder,
    diagnostics::LocationInfo,
    graph::Graph,
    ir::{
        Attribute, AttributeValue, BasicBlock, Intrinsic, IntrinsicTrait, Operation,
        SupportsVerification, Var,
    },
    pass_manager::{AnalysisManager, OperationPass, OperationPassManager, PassManager},
    region::Region,
    ssacfg::SSACFG,
};
