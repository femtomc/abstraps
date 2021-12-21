//! The design of this IR is a close copy of MLIR and
//! can be thought of as an embedding of MLIR concepts in Rust.
//! This IR uses parametrized basic blocks (in contrast to phi nodes).
//! The core of the IR is the `Operation` template.
//!
//! The implementation reflects the extensible design of MLIR.
//! This IR can be thought of as a stage which can further target dialects of MLIR.
//!
//! For further information on SSA-based IRs:
//! https://en.wikipedia.org/wiki/Static_single_assignment_form
//! for more background on SSA.

use crate::core::diagnostics::LocationInfo;
use crate::core::interfaces::*;
use crate::core::region::Region;
use alloc::string::String;
use alloc::vec::Vec;
use color_eyre::{eyre::bail, Report};
use downcast_rs::{impl_downcast, Downcast};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Var(usize);

impl Var {
    pub fn new(id: usize) -> Var {
        Var(id)
    }

    pub fn get_id(&self) -> usize {
        self.0
    }
}

pub trait Intrinsic: Downcast + Object + ObjectClone {
    fn get_namespace(&self) -> &str;
    fn get_name(&self) -> &str;
    fn get_unique_id(&self) -> String {
        format!("{}.{}", self.get_namespace(), self.get_name())
    }
}
impl_downcast!(Intrinsic);
mopo!(dyn Intrinsic);

pub trait Attribute: Object + std::fmt::Display {}
mopo!(dyn Attribute);

pub trait AttributeValue<T> {
    fn get_value(&self) -> &T;
    fn get_value_mut(&mut self) -> &mut T;
}

pub trait SupportsInterfaceTraits {
    fn get_intrinsic(&self) -> &Box<dyn Intrinsic>;
    fn get_regions(&self) -> &[Region];
    fn get_attributes(&self) -> &HashMap<String, Box<dyn Attribute>>;
    fn get_attributes_mut(&mut self) -> &mut HashMap<String, Box<dyn Attribute>>;
}

#[derive(Debug)]
pub struct Operation {
    location: LocationInfo,
    intrinsic: Box<dyn Intrinsic>,
    operands: Vec<Var>,
    attributes: HashMap<String, Box<dyn Attribute>>,
    regions: Vec<Region>,
    successors: Vec<BasicBlock>,
}

impl Hash for Operation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.location.hash(state);
        self.intrinsic.hash(state);
        self.operands.hash(state);
        self.regions.hash(state);
        self.successors.hash(state);
    }
}

impl SupportsInterfaceTraits for Operation {
    fn get_intrinsic(&self) -> &Box<dyn Intrinsic> {
        &self.intrinsic
    }

    fn get_regions(&self) -> &[Region] {
        &self.regions
    }

    fn get_attributes(&self) -> &HashMap<String, Box<dyn Attribute>> {
        &self.attributes
    }

    fn get_attributes_mut(&mut self) -> &mut HashMap<String, Box<dyn Attribute>> {
        &mut self.attributes
    }
}

impl Operation {
    pub fn new(
        location: LocationInfo,
        intrinsic: Box<dyn Intrinsic>,
        operands: Vec<Var>,
        attributes: HashMap<String, Box<dyn Attribute>>,
        regions: Vec<Region>,
        successors: Vec<BasicBlock>,
    ) -> Operation {
        Operation {
            location,
            intrinsic,
            operands,
            attributes,
            regions,
            successors,
        }
    }

    pub fn get_location(&self) -> &LocationInfo {
        &self.location
    }

    pub fn get_operands(&self) -> Vec<Var> {
        self.operands.to_vec()
    }
}

impl Operation {
    pub fn get_attributes(&self) -> &HashMap<String, Box<dyn Attribute>> {
        &self.attributes
    }

    pub fn get_attributes_mut(&mut self) -> &mut HashMap<String, Box<dyn Attribute>> {
        &mut self.attributes
    }
}

#[derive(Debug, Hash)]
pub struct BasicBlock {
    operands: Vec<Var>,
    ops: Vec<Operation>,
}

impl Default for BasicBlock {
    fn default() -> BasicBlock {
        BasicBlock {
            ops: Vec::new(),
            operands: Vec::new(),
        }
    }
}

impl BasicBlock {
    pub fn get_ops(&self) -> &[Operation] {
        &self.ops
    }

    pub fn get_ops_mut(&mut self) -> &mut Vec<Operation> {
        &mut self.ops
    }

    pub fn get_operands(&self) -> &[Var] {
        &self.operands
    }

    pub fn get_operands_mut(&mut self) -> &mut Vec<Var> {
        &mut self.operands
    }
}

/////
///// Lowering.
/////

/// Defines the interfaces by which an AST can target
/// and lower to an IR of type `T`.
pub trait Lowering<T> {
    type IRBuilder;
    type Error;
    fn prepare(&self) -> Result<Self::IRBuilder, Self::Error>;
    fn build(&self, b: &mut Self::IRBuilder) -> Result<(), Self::Error>;
    fn lower(&self) -> Result<T, Self::Error>;
}
