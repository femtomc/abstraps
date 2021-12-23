//! The design of this IR is a close copy of MLIR and
//! can be thought of as an embedding of MLIR concepts in Rust.
//! This IR uses parametrized basic blocks (in contrast to phi nodes).
//! The core of the IR is the `Operation` template.
//!
//! The implementation reflects the extensible design of MLIR.
//! This IR can be thought of as a stage which can further target dialects of MLIR.
//!
//! For further information on SSA-based IRs:
//! `<https://en.wikipedia.org/wiki/Static_single_assignment_form>`
//! for more background on SSA.

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
    absint::{LatticeJoin, LatticeSemantics, Signature},
    builder::OperationBuilder,
    diagnostics::{diagnostics_setup, LocationInfo},
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
