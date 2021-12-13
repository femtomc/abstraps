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

//use crate::ir::graph::Graph;
//use crate::ir::ssacfg::SSACFG;
use alloc::string::String;
use alloc::vec::Vec;
use anyhow;
use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use {indenter::indented, std::fmt::Write};

#[derive(Clone, Debug)]
pub enum IRError {
    Fallback,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IRLInfo {
    file: String,
    module: String,
    line: usize,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

pub trait Intrinsic
where
    Self: std::fmt::Debug,
{
    fn get_namespace(&self) -> &str;
    fn get_name(&self) -> &str;
    fn get_traits(&self) -> Vec<Box<dyn OperationTrait>>;
}

pub trait OperationTrait
where
    Self: std::fmt::Debug,
{
    fn verify(&self, op: &Operation) -> anyhow::Result<()>;
}

pub trait AttributeValue
where
    Self: std::fmt::Debug,
{
}

impl<T> AttributeValue for T where T: std::fmt::Debug {}

pub trait Attribute
where
    Self: std::fmt::Debug,
{
    fn get_value<T>(&self) -> Box<T>;
}

#[derive(Debug)]
pub struct Operation {
    intr: Box<dyn Intrinsic>,
    args: Vec<Var>,
    attrs: HashMap<String, Box<dyn Attribute>>,
    regions: Vec<Region>,
    successors: Vec<BasicBlock>,
}

#[derive(Debug)]
pub struct BasicBlock {
    args: Vec<Var>,
    ops: Vec<Operation>,
}

/// A close copy of the equivalent concept in MLIR.
///
/// A region represents a scope controlled by the parent operation.
/// The scope itself can have various attributes applied to it
/// (in MLIR, this is via the trait system).
#[derive(Debug)]
pub enum Region {
    //    Directed(SSACFG),
//    Undirected(Graph),
}

/////
///// Operation verification.
/////

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
