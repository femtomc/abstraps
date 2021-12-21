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

pub trait IntrinsicTrait: Downcast
where
    Self: std::fmt::Debug,
{
    fn verify(&self, op: &dyn SupportsVerification) -> Result<(), Report>;

    fn get_attribute<'a>(&self, _op: &'a Operation) -> Result<&'a Box<dyn Attribute>, Report> {
        bail!(format!(
            "(Fallback) Failed to get attribute associated with {:?}.",
            self
        ))
    }

    fn get_attribute_mut<'a>(
        &self,
        _op: &'a mut Operation,
    ) -> Result<&'a mut Box<dyn Attribute>, Report> {
        bail!(format!(
            "(Fallback) Failed to get attribute associated with {:?}.",
            self
        ))
    }
}
impl_downcast!(IntrinsicTrait);

pub trait Intrinsic
where
    Self: Downcast + std::fmt::Debug,
{
    fn get_namespace(&self) -> &str;
    fn get_name(&self) -> &str;
    fn get_traits(&self) -> Vec<Box<dyn IntrinsicTrait>>;
    fn get_unique_id(&self) -> String {
        format!("{}.{}", self.get_namespace(), self.get_name())
    }
}
impl_downcast!(Intrinsic);

impl Hash for dyn Intrinsic {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let s = format!("{}.{}", self.get_namespace(), self.get_name());
        s.hash(state)
    }
}

pub trait AttributeValue
where
    Self: std::fmt::Debug,
{
}
impl<T> AttributeValue for T where T: std::fmt::Debug {}

pub trait Attribute: Downcast + std::fmt::Display
where
    Self: std::fmt::Debug,
{
    fn get_value(&self) -> &dyn AttributeValue;
    fn get_value_mut(&mut self) -> &mut dyn AttributeValue;
}
impl_downcast!(Attribute);

pub trait SupportsVerification
where
    Self: std::fmt::Debug,
{
    fn get_intrinsic(&self) -> &Box<dyn Intrinsic>;
    fn get_attributes(&self) -> &HashMap<String, Box<dyn Attribute>>;
    fn get_regions(&self) -> &[Region];
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

impl SupportsVerification for Operation {
    fn get_intrinsic(&self) -> &Box<dyn Intrinsic> {
        &self.intrinsic
    }

    fn get_attributes(&self) -> &HashMap<String, Box<dyn Attribute>> {
        &self.attributes
    }

    fn get_regions(&self) -> &[Region] {
        &self.regions
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

    // This is absolutely crazy that this is required -
    // but for code which looks at `Operation`, you can't make any
    // trait statements (because of the dynamism, no generics).
    // So here, what this is doing is saying - give me an `IntrinsicTrait`
    // type, I'm going to ask the `dyn Intrinsic` in `Operation`
    // for all the `IntrinsicTrait` instances the operation is supposed
    // to support. Then, it tries to downcast each one to
    // the `IntrinsicTrait` type - and if it succeeds,
    // it will use the associated `IntrinsicTrait` method `verify`
    // to try and `verify` that the operation does indeed satisfy
    // the `IntrinsicTrait`.
    //
    // This makes use of `downcast_rs` -- and what I assume is complete
    // wizardry.
    pub fn check_trait<K>(&self) -> Option<Result<(), Report>>
    where
        K: IntrinsicTrait,
    {
        self.get_intrinsic()
            .get_traits()
            .iter()
            .find_map(|tr| tr.downcast_ref::<K>().map(|v| v.verify(self)))
    }

    pub fn has_trait<K>(&self) -> bool
    where
        K: IntrinsicTrait,
    {
        match self.check_trait::<K>() {
            Some(v) => v.is_ok(),
            None => false,
        }
    }

    pub fn get_trait<K>(&self) -> Result<Box<K>, Report>
    where
        K: IntrinsicTrait + Copy,
    {
        let tr = self
            .get_intrinsic()
            .get_traits()
            .into_iter()
            .find(|v| v.is::<K>());
        match tr {
            None => bail!("Failed to get trait."),
            Some(v) => Ok(v.downcast::<K>().unwrap()),
        }
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
