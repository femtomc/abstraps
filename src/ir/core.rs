/*!

  The design of this IR is heavily influenced by
  WebAssembly, Julia's IRTools IR, Julia's IRCode IR,
  and MLIR. It follows the latter (MLIR) most closely,
  and can be thought of as a simple embedding of MLIR concepts in Rust.

  This IR uses parametrized basic blocks (in contrast to phi nodes).

  The core of the IR is the `Operation` template.

  This reflects inspiration from the extensible design of MLIR,
  (but conversion between "intrinsic dialects" is out of scope)
  This IR can be thought of as a stage which can
  further target dialects of MLIR.

  The IR intrinsics and attributes are generic (denoted by `I`
  and `A` in the code below) - downstream dependents
  should define their own set of intrinsics and attributes,
  and can then define their own lowering,
  abstract interpretation, and code generation.

  For further information on SSA-based IRs:
  https://en.wikipedia.org/wiki/Static_single_assignment_form
  for more background on SSA.

*/

use crate::ir::builder::OperationBuilder;
use crate::ir::graph::Graph;
use crate::ir::ssacfg::SSACFG;
use alloc::string::String;
use alloc::vec::Vec;
use anyhow;
use anyhow::bail;
use downcast_rs::{impl_downcast, Downcast};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use {indenter::indented, std::fmt::Write};

#[derive(Clone, Debug)]
pub enum IRError {
    Fallback,
}

#[derive(Clone, Debug)]
pub struct IRLInfo {
    file: String,
    module: String,
    line: usize,
}

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

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "%{}", self.get_id())
    }
}

pub trait IntrinsicTrait: Downcast
where
    Self: std::fmt::Debug,
{
    fn verify(&self, op: &dyn SupportsVerification) -> anyhow::Result<()>;
    fn get_attribute_mut<'a>(
        &self,
        op: &'a mut OperationBuilder,
    ) -> anyhow::Result<&'a mut Box<dyn Attribute>> {
        bail!(format!(
            "Failed to get attribute associated with {:?}.",
            self
        ))
    }
}
impl_downcast!(IntrinsicTrait);

pub trait Intrinsic
where
    Self: std::fmt::Debug,
{
    fn get_namespace(&self) -> &str;
    fn get_name(&self) -> &str;
    fn get_traits(&self) -> Vec<Box<dyn IntrinsicTrait>>;
    fn get_builder(&self) -> OperationBuilder;
}

impl fmt::Display for dyn Intrinsic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.get_namespace(), self.get_name())
    }
}

pub trait AttributeValue
where
    Self: std::fmt::Debug,
{
}

impl fmt::Display for dyn AttributeValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<T> AttributeValue for T where T: std::fmt::Debug {}

pub trait Attribute: Downcast
where
    Self: std::fmt::Debug,
{
    fn get_value(&self) -> &dyn AttributeValue;
    fn get_value_mut(&mut self) -> &mut dyn AttributeValue;
}
impl_downcast!(Attribute);

impl fmt::Display for dyn Attribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = self.get_value();
        write!(f, "{:?}", v)
    }
}

pub trait SupportsVerification
where
    Self: std::fmt::Debug,
{
    fn get_intrinsic(&self) -> &Box<dyn Intrinsic>;
    fn get_attributes(&self) -> &HashMap<String, Box<dyn Attribute>>;
}

#[derive(Debug)]
pub struct Operation {
    intrinsic: Box<dyn Intrinsic>,
    operands: Vec<Var>,
    attributes: HashMap<String, Box<dyn Attribute>>,
    regions: Vec<Region>,
    successors: Vec<BasicBlock>,
}

impl SupportsVerification for Operation {
    fn get_intrinsic(&self) -> &Box<dyn Intrinsic> {
        &self.intrinsic
    }

    fn get_attributes(&self) -> &HashMap<String, Box<dyn Attribute>> {
        return &self.attributes;
    }
}

impl Operation {
    pub fn new(
        intrinsic: Box<dyn Intrinsic>,
        operands: Vec<Var>,
        attributes: HashMap<String, Box<dyn Attribute>>,
        regions: Vec<Region>,
        successors: Vec<BasicBlock>,
    ) -> Operation {
        Operation {
            intrinsic,
            operands,
            attributes,
            regions,
            successors,
        }
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
    pub fn check_trait<K>(&self) -> Option<anyhow::Result<()>>
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

    pub fn get_trait<K>(&self) -> anyhow::Result<Box<K>>
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
    pub fn get_attributes_mut(&mut self) -> &mut HashMap<String, Box<dyn Attribute>> {
        return &mut self.attributes;
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.intrinsic)?;
        if !self.operands.is_empty() {
            write!(f, "(")?;
            let l = self.operands.len();
            for (ind, arg) in self.operands.iter().enumerate() {
                match l - 1 == ind {
                    true => write!(f, "{}", arg)?,
                    _ => write!(f, "{}, ", arg)?,
                };
            }
            write!(f, ")")?;
        }
        if !self.attributes.is_empty() {
            write!(f, " {{ ")?;
            let l = self.attributes.len();
            for (ind, attr) in self.attributes.iter().enumerate() {
                match l - 1 == ind {
                    true => write!(f, "{} = {}", attr.0, attr.1)?,
                    _ => write!(f, "{} = {}, ", attr.0, attr.1)?,
                };
            }
            write!(f, " }}")?;
        }
        if !self.regions.is_empty() {
            for r in self.regions.iter() {
                writeln!(f, " {{")?;
                write!(indented(f).with_str("  "), "{}", r)?;
                write!(f, "}}")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
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

/// A close copy of the equivalent concept in MLIR.
///
/// A region represents a scope controlled by the parent operation.
/// The scope itself can have various attributes applied to it
/// (in MLIR, this is via the trait system).
#[derive(Debug)]
pub enum Region {
    Directed(SSACFG),
    Undirected(Graph),
}

impl Region {
    /// Get an immutable iterator over basic blocks.
    fn block_iter(&self, id: usize) -> ImmutableBlockIterator {
        let ks = match self {
            Region::Directed(ssacfg) => ssacfg.get_block_vars(id),
            Region::Undirected(graph) => graph.get_block_vars(),
        };
        ImmutableBlockIterator {
            region: self,
            ks,
            state: 0,
        }
    }

    pub fn push_arg(&mut self, ind: usize) -> anyhow::Result<Var> {
        match self {
            Region::Directed(ssacfg) => Ok(ssacfg.push_arg(ind)),
            Region::Undirected(graph) => {
                bail!("Can't push argument onto `Graph` region.")
            }
        }
    }

    pub fn push_op(&mut self, blk: usize, op: Operation) -> Var {
        match self {
            Region::Directed(ssacfg) => ssacfg.push_op(blk, op),
            Region::Undirected(graph) => graph.push_op(op),
        }
    }

    pub fn get_op(&self, id: Var) -> Option<(Var, &Operation)> {
        match self {
            Region::Directed(ssacfg) => ssacfg.get_op(id),
            Region::Undirected(graph) => graph.get_op(id),
        }
    }

    pub fn push_block(&mut self, b: BasicBlock) -> anyhow::Result<()> {
        match self {
            Region::Directed(ssacfg) => {
                ssacfg.push_block(b);
                Ok(())
            }
            Region::Undirected(graph) => {
                if graph.has_block() {
                    bail!("Can't push block onto `Graph` region which already has a block.")
                } else {
                    graph.push_block(b);
                    Ok(())
                }
            }
        }
    }

    pub fn get_block(&mut self, ind: usize) -> &mut BasicBlock {
        match self {
            Region::Directed(ssacfg) => ssacfg.get_block(ind),
            Region::Undirected(graph) => graph.get_block(),
        }
    }
}

#[derive(Debug)]
pub struct ImmutableBlockIterator<'b> {
    region: &'b Region,
    ks: Vec<Var>,
    state: usize,
}

impl<'b> Iterator for ImmutableBlockIterator<'b> {
    type Item = (Var, &'b Operation);

    fn next(&mut self) -> Option<Self::Item> {
        if self.ks.len() > self.state {
            let p = self.region.get_op(self.ks[self.state]);
            self.state += 1;
            return p;
        }
        None
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Region::Directed(ssacfg) => {
                for ind in 0..ssacfg.get_blocks().len() {
                    write!(f, "{}: ", ind)?;
                    let b = &ssacfg.get_blocks()[ind];
                    let boperands = &b.get_operands();
                    if !boperands.is_empty() {
                        write!(f, "(")?;
                        let l = boperands.len();
                        for (ind, arg) in boperands.iter().enumerate() {
                            match l - 1 == ind {
                                true => write!(f, "{}", arg)?,
                                _ => write!(f, "{}, ", arg)?,
                            };
                        }
                        write!(f, ")")?;
                    }
                    writeln!(f)?;
                    for (v, op) in self.block_iter(ind) {
                        writeln!(indented(f).with_str("  "), "{} = {}", v, op)?;
                    }
                }
                Ok(())
            }

            Region::Undirected(graph) => {
                for (v, op) in self.block_iter(0) {
                    writeln!(indented(f).with_str("  "), "{} = {}", v, op)?;
                }
                Ok(())
            }
        }
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

/////
///// Abstract interpreter.
/////

/*

   The purpose of an abstract interpreter is
   to virtually interpret the IR on a lattice
   (over concrete data flow) defined by abstraction function A
   and concretization function C.

   IR intrinsics are defined as primitives
   for the interpretation. Then, other functions which are
   defined using the primitives have "derived" interpretations.

   The interface below is relatively open -- not much guidance
   is provided for satisfying the interface, other than
   that the methods (as specified) roughly represent
   the process of preparing an inhabitant of `AbstractInterpreter`
   from a piece of IR, repeatedly applying a transition function
   (here: `step`), and then returning a result.

*/

/// The `AbstractInterpreter<IR, R>` trait specifies an interface
/// for the implementation of abstract interpreters which operate
/// on intermediate representations of type `IR`, and return a result
/// artifact of type `R`.
///
/// Interpreters are prepared using "reference information" (e.g. a type mapping from arguments to
/// types, etc) whose type is specified with `Self::Meta`, and an immutable pointer to the `IR` (so
/// that interpreters can prepare block frames, etc).
///
/// Interpreters can error inside of method calls with type `Self::Error`.
pub trait AbstractInterpreter<IR, R> {
    /// Meta information required to prepare an interpreter.
    type Meta;

    /// The type of lattice elements.
    type LatticeElement;

    /// An associated error type for the implementor.
    type Error;

    /// `prepare` accepts a `Self::Meta` instance and a reference
    /// to an `IR` type instance and prepares an interpreter.
    ///
    /// The `Self::Meta` type can be a type signature, or other
    /// piece of meta-information.
    fn prepare(meta: Self::Meta, ir: &IR) -> Result<Self, Self::Error>
    where
        Self: Sized;

    /// `step` performs a single abstract interpretation step
    /// with reference to the `ir: &IR`. The `step` may fail -- returning
    /// a `Err(Self::Error)` value. Otherwise, the `step` will
    /// likely mutate the interpreter -- progressing its analysis
    /// and storing any interpretation metadata.
    fn step(&mut self, ir: &IR) -> Result<(), Self::Error>;

    /// `finish` produces a result artifact `R` and resets
    /// the interpreter. This could include mutating the state
    /// of the interpreter (resetting fields) -- this
    /// method will likely be called before the interpreter
    /// is dropped.
    fn finish(&mut self) -> Result<R, Self::Error>;
}
