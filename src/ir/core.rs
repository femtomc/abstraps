/*!

  The design of this IR is heavily influenced by
  WebAssembly, Julia's IRTools IR, Julia's IRCode IR,
  and MLIR. It follows the latter (MLIR) most closely,
  and can be thought of as a simple embedding of MLIR concepts in Rust.

  This IR uses parametrized basic blocks (in contrast to phi nodes).

  The core of the IR is the `Operation<I, A>` template,
  where `I` denotes the set of intrinsics (user-defined)
  and `A` denotes the set of attributes (static information
  about operations) which is also user-defined.

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

use crate::ir::graph::Graph;
use crate::ir::ssacfg::SSACFG;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Operation<I, A> {
    intr: I,
    args: Vec<Var>,
    attrs: HashMap<String, A>,
    regions: Vec<Region<I, A>>,
    successors: Vec<BasicBlock<I, A>>,
}

impl<I, A> Operation<I, A> {
    pub fn new(
        intr: I,
        args: Vec<Var>,
        attrs: HashMap<String, A>,
        regions: Vec<Region<I, A>>,
        successors: Vec<BasicBlock<I, A>>,
    ) -> Operation<I, A> {
        Operation {
            intr,
            args,
            attrs,
            regions,
            successors,
        }
    }

    pub fn get_intrinsic(&self) -> &I {
        &self.intr
    }

    pub fn get_args(&self) -> &[Var] {
        &self.args
    }

    pub fn get_attr(&self, key: &str) -> Option<&A> {
        self.attrs.get(key)
    }

    pub fn get_attrs(&self) -> &HashMap<String, A> {
        &self.attrs
    }

    pub fn get_regions(&self) -> &[Region<I, A>] {
        &self.regions
    }

    pub fn get_successors(&self) -> &[BasicBlock<I, A>] {
        &self.successors
    }
}

impl<I1, A1> Operation<I1, A1> {
    pub fn pass<R>(&self, f: &dyn Fn(&Operation<I1, A1>) -> R, accum: &dyn Fn(Vec<R>) -> R) -> R {
        let s = accum(
            self.successors
                .iter()
                .map(|s| s.pass(f, accum))
                .collect::<Vec<_>>(),
        );
        let r = accum(
            self.regions
                .iter()
                .map(|r| r.pass(f, accum))
                .collect::<Vec<_>>(),
        );
        return accum(vec![f(self), s, r]);
    }

    pub fn bifmap<I2, A2>(
        mut self,
        fintr: &dyn Fn(I1) -> I2,
        fattr: &dyn Fn(A1) -> A2,
    ) -> Operation<I2, A2> {
        let intr = fintr(self.intr);
        let attrs = self
            .attrs
            .drain()
            .map(|(k, v)| (k, fattr(v)))
            .collect::<HashMap<_, _>>();
        let regions = self
            .regions
            .into_iter()
            .map(|r| r.bifmap(fintr, fattr))
            .collect::<Vec<Region<_, _>>>();
        let successors = self
            .successors
            .into_iter()
            .map(|s| s.bifmap(fintr, fattr))
            .collect::<Vec<BasicBlock<_, _>>>();
        Operation::<I2, A2>::new(intr, self.args, attrs, regions, successors)
    }
}

impl<I, A> fmt::Display for Operation<I, A>
where
    I: fmt::Display,
    A: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.intr)?;
        if !self.args.is_empty() {
            write!(f, "(")?;
            let l = self.args.len();
            for (ind, arg) in self.args.iter().enumerate() {
                match l - 1 == ind {
                    true => write!(f, "{}", arg)?,
                    _ => write!(f, "{}, ", arg)?,
                };
            }
            write!(f, ")")?;
        }
        if !self.attrs.is_empty() {
            write!(f, " {{ ")?;
            let l = self.attrs.len();
            for (ind, attr) in self.attrs.iter().enumerate() {
                match l - 1 == ind {
                    true => write!(f, "{} = {}", attr.0, attr.1)?,
                    _ => write!(f, "{} = {}, ", attr.0, attr.1)?,
                };
            }
            write!(f, " }}")?;
        }
        if !self.regions.is_empty() {
            for r in self.regions.iter() {
                write!(f, " {{\n")?;
                write!(indented(f).with_str("  "), "{}", r)?;
                write!(f, "}}")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BasicBlock<I, A> {
    args: Vec<Var>,
    ops: Vec<Operation<I, A>>,
}

impl<I, A> BasicBlock<I, A> {
    pub fn get_ops(&self) -> &[Operation<I, A>] {
        &self.ops
    }

    pub fn get_ops_mut(&mut self) -> &mut Vec<Operation<I, A>> {
        &mut self.ops
    }

    pub fn get_args(&self) -> &[Var] {
        &self.args
    }

    pub fn get_args_mut(&mut self) -> &mut Vec<Var> {
        &mut self.args
    }
}

impl<I, A> Default for BasicBlock<I, A> {
    fn default() -> BasicBlock<I, A> {
        BasicBlock {
            ops: Vec::new(),
            args: Vec::new(),
        }
    }
}

impl<I1, A1> BasicBlock<I1, A1> {
    pub fn pass<R>(&self, f: &dyn Fn(&Operation<I1, A1>) -> R, accum: &dyn Fn(Vec<R>) -> R) -> R {
        let ops = self
            .ops
            .iter()
            .map(|op| op.pass(f, accum))
            .collect::<Vec<_>>();
        accum(ops)
    }

    pub fn bifmap<I2, A2>(
        mut self,
        fintr: &dyn Fn(I1) -> I2,
        fattr: &dyn Fn(A1) -> A2,
    ) -> BasicBlock<I2, A2> {
        let ops = self
            .ops
            .into_iter()
            .map(|op| op.bifmap(fintr, fattr))
            .collect::<Vec<_>>();
        BasicBlock {
            ops,
            args: self.args,
        }
    }
}

/// A close copy of the equivalent concept in MLIR.
///
/// A region represents a scope controlled by the parent operation.
/// The scope itself can have various attributes applied to it
/// (in MLIR, this is via the trait system).
#[derive(Debug, Serialize, Deserialize)]
pub enum Region<I, A> {
    Directed(SSACFG<I, A>),
    Undirected(Graph<I, A>),
}

impl<I, A> Region<I, A> {
    /// Get an immutable iterator over basic blocks.
    fn block_iter(&self, id: usize) -> ImmutableBlockIterator<I, A> {
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

    pub fn push_op(&mut self, blk: usize, op: Operation<I, A>) -> Var {
        match self {
            Region::Directed(ssacfg) => ssacfg.push_op(blk, op),
            Region::Undirected(graph) => graph.push_op(op),
        }
    }

    pub fn get_op(&self, id: Var) -> Option<(Var, &Operation<I, A>)> {
        match self {
            Region::Directed(ssacfg) => ssacfg.get_op(id),
            Region::Undirected(graph) => graph.get_op(id),
        }
    }

    pub fn push_block(&mut self, b: BasicBlock<I, A>) -> anyhow::Result<()> {
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

    pub fn get_block(&mut self, ind: usize) -> &mut BasicBlock<I, A> {
        match self {
            Region::Directed(ssacfg) => ssacfg.get_block(ind),
            Region::Undirected(graph) => graph.get_block(),
        }
    }
}

impl<I, A> fmt::Display for Region<I, A>
where
    I: fmt::Display,
    A: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Region::Directed(ssacfg) => {
                for ind in 0..ssacfg.get_blocks().len() {
                    write!(f, "{}: ", ind)?;
                    let b = &ssacfg.get_blocks()[ind];
                    let bargs = &b.get_args();
                    if !bargs.is_empty() {
                        write!(f, "(")?;
                        let l = bargs.len();
                        for (ind, arg) in bargs.iter().enumerate() {
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

impl<I1, A1> Region<I1, A1> {
    pub fn pass<R>(&self, f: &dyn Fn(&Operation<I1, A1>) -> R, accum: &dyn Fn(Vec<R>) -> R) -> R {
        match self {
            Region::Directed(r) => r.pass(f, accum),
            Region::Undirected(r) => r.pass(f, accum),
        }
    }

    pub fn bifmap<I2, A2>(
        mut self,
        fintr: &dyn Fn(I1) -> I2,
        fattr: &dyn Fn(A1) -> A2,
    ) -> Region<I2, A2> {
        match self {
            Region::Directed(r) => Region::Directed(r.bifmap(fintr, fattr)),
            Region::Undirected(r) => Region::Undirected(r.bifmap(fintr, fattr)),
        }
    }
}

#[derive(Debug)]
pub struct ImmutableBlockIterator<'b, I, A> {
    region: &'b Region<I, A>,
    ks: Vec<Var>,
    state: usize,
}

impl<'b, I, A> Iterator for ImmutableBlockIterator<'b, I, A> {
    type Item = (Var, &'b Operation<I, A>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.ks.len() > self.state {
            let p = self.region.get_op(self.ks[self.state]);
            self.state += 1;
            return p;
        }
        None
    }
}

/////
///// Operation verification.
/////

pub trait Verify
where
    Self: Sized,
{
    fn verify<A>(
        &self,
        args: Vec<Var>,
        attrs: HashMap<String, A>,
        regions: Vec<Region<Self, A>>,
        successors: Vec<BasicBlock<Self, A>>,
    ) -> anyhow::Result<()>;
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
