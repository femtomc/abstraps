/*

   This file is part of `abstraps`. License is MIT.

   The design of this IR is heavily influenced by
   WebAssembly, Julia's IRTools IR, Julia's IRCode IR,
   and MLIR.

   This IR uses parametrized basic blocks (in contrast to phi nodes).

   The core of the IR is the `Instruction<I, A>` template,
   where `T` denotes the set of intrinsics (user-defined)
   and `K` denotes the set of attributes (static information
   about instructions) which is also user-defined.

   This reflects inspiration from  the extensible design of MLIR,
   (but conversion between "intrinsic dialects" is out of scope)
   This IR can be thought of as a stage which can
   further target dialects of MLIR.

   The IR intrinsics and attributes are generic (denoted by T
   and K in the code below) -- downstream dependents
   should define their own set of intrinsics and attributes,
   and can then define their own lowering,
   abstract interpretation, and code generation.

   Please read:
   https://en.wikipedia.org/wiki/Static_single_assignment_form
   for more background on SSA.

*/

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub enum IRError {
    Fallback,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Var(usize);

impl Var {
    pub fn get_id(&self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Operator<I> {
    Intrinsic(I),
    ModuleRef(Option<String>, String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Instruction<I, A> {
    op: Operator<I>,
    args: Vec<Var>,
    attrs: Vec<A>,
}

impl<I, A> Instruction<I, A> {
    pub fn new(op: Operator<I>, args: Vec<Var>, attrs: Vec<A>) -> Instruction<I, A> {
        Instruction { op, args, attrs }
    }

    pub fn get_op(&self) -> &Operator<I> {
        &self.op
    }

    pub fn get_args(&self) -> &[Var] {
        &self.args
    }

    pub fn get_attrs(&self) -> &[A] {
        &self.attrs
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Branch {
    cond: Option<Var>,
    block: usize,
    args: Vec<Var>,
}

impl Branch {
    pub fn get_args(&self) -> &[Var] {
        &self.args
    }

    pub fn get_block(&self) -> usize {
        self.block
    }

    pub fn is_conditional(&self) -> bool {
        match self.cond {
            Some(_v) => true,
            None => false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BasicBlock<I, A> {
    args: Vec<Var>,
    insts: Vec<Instruction<I, A>>,
    branches: Vec<Branch>,
}

impl<I, A> BasicBlock<I, A> {
    pub fn get_insts(&self) -> &[Instruction<I, A>] {
        &self.insts
    }

    pub fn get_args(&self) -> &[Var] {
        &self.args
    }

    pub fn get_branches(&self) -> &[Branch] {
        &self.branches
    }
}

impl<I, A> Default for BasicBlock<I, A> {
    fn default() -> BasicBlock<I, A> {
        BasicBlock {
            insts: Vec::new(),
            args: Vec::new(),
            branches: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IRLInfo {
    file: String,
    module: String,
    line: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtIR<I, A> {
    defs: Vec<(i32, i32)>,
    lines: Vec<Option<IRLInfo>>,
    blocks: Vec<BasicBlock<I, A>>,
}

impl<I, A> Default for ExtIR<I, A> {
    fn default() -> ExtIR<I, A> {
        let entry = BasicBlock::default();
        ExtIR {
            defs: Vec::new(),
            blocks: vec![entry],
            lines: Vec::new(),
        }
    }
}

impl<I, A> ExtIR<I, A> {
    pub fn get_args(&self) -> &[Var] {
        self.blocks[0].get_args()
    }

    pub fn get_block_args(&self, blk: usize) -> &[Var] {
        self.blocks[blk].get_args()
    }

    pub fn push_arg(&mut self, blk: usize) -> Var {
        let arg = Var(self.defs.len());
        self.defs.push((blk as i32, -1));
        self.blocks[blk].args.push(arg);
        arg
    }

    pub fn push_blk(&mut self) -> usize {
        let blk = BasicBlock::default();
        self.blocks.push(blk);
        self.blocks.len() - 1
    }

    pub fn get_branches(&self, blk: usize) -> &[Branch] {
        &self.blocks[blk].branches
    }

    pub fn get_branches_mut(&mut self, blk: usize) -> &mut Vec<Branch> {
        &mut self.blocks[blk].branches
    }

    /// Push a branch from the block indexed by `from`
    /// to the branch indexed by `to`.
    /// The user must supply a branch condition in `cond`.
    pub fn push_branch(&mut self, cond: Option<Var>, from: usize, to: usize, args: Vec<Var>) {
        let brs = self.get_branches_mut(from);
        match cond {
            None => brs.retain(|x| x.is_conditional()),
            Some(_v) => (),
        }
        brs.push(Branch {
            cond,
            block: to,
            args,
        });
    }

    /// Get an immutable reference to a "line" of the IR.
    /// The IR is indexed with `id` (a `Var` instance).
    /// This returns `(Var, &Instruction<I, A>)`.
    pub fn get_instr(&self, id: Var) -> Option<(Var, &Instruction<I, A>)> {
        match self.get_var_blockidx(id) {
            None => None,
            Some((b, i)) => {
                let bb = &self.blocks[b];
                let inst = &bb.insts[i as usize];
                Some((id, inst))
            }
        }
    }

    /// Push an instruction onto the IR at block index `blk`.
    /// Returns a new `Var` reference to that instruction.
    pub fn push_instr(&mut self, blk: usize, v: Instruction<I, A>) -> Var {
        let arg = Var(self.defs.len());
        let len = self.blocks[blk].insts.len();
        let bb = &mut self.blocks[blk];
        bb.insts.push(v);
        self.defs.push((blk as i32, len as i32));
        arg
    }

    fn get_instr_mut(&mut self, id: Var) -> Option<(Var, &mut Instruction<I, A>)> {
        match self.get_var_blockidx(id) {
            None => None,
            Some((b, i)) => {
                let bb = &mut self.blocks[b];
                let inst = &mut bb.insts[i as usize];
                Some((id, inst))
            }
        }
    }

    /// Get the vector of `Var` which index into block with index `id`.
    fn get_block_vars(&self, id: usize) -> Vec<Var> {
        let v = self
            .defs
            .iter()
            .enumerate()
            .filter(|(_, v)| v.0 == (id as i32) && v.1 >= 0);
        let mut m = v.map(|(i, l)| (Var(i), l)).collect::<Vec<_>>();
        m.sort_by(|a, b| a.1.cmp(b.1));
        m.iter().map(|v| v.0).collect::<Vec<_>>()
    }

    /// Get an immutable iterator over basic blocks.
    fn block_iter(&self, id: usize) -> ImmutableBlockIterator<I, A> {
        let ks = self.get_block_vars(id);
        ImmutableBlockIterator {
            ir: self,
            ks,
            state: 0,
        }
    }

    /// Get a mutable iterator over basic blocks.
    fn block_iter_mut(&mut self, id: usize) -> MutableBlockIterator<I, A> {
        let ks = self.get_block_vars(id);
        MutableBlockIterator {
            ir: self,
            ks,
            state: 0,
        }
    }

    /// Get the block index and SSA index for `v: Var`.
    fn get_var_blockidx(&self, v: Var) -> Option<(usize, i32)> {
        let (b, i) = self.defs.get(v.get_id()).unwrap_or(&(-1, -1));
        if *i < 0 {
            None
        } else {
            Some((*b as usize, *i))
        }
    }

    pub fn get_vars_in_block(&self, blockidx: usize) -> Vec<Var> {
        let mut v: Vec<Var> = Vec::new();
        for ind in 0..self.defs.len() {
            let r = Var(ind);
            match self.get_var_blockidx(r) {
                None => (),
                Some(b) => {
                    if b.0 == blockidx && self.defs[ind].1 >= 0 {
                        v.push(r);
                    }
                }
            }
        }
        v
    }

    fn delete_block(&mut self, i: usize) {
        self.blocks.remove(i);
        if i != self.blocks.len() + 1 {
            for b in self.blocks.iter_mut() {
                for bi in 1..b.branches.len() {
                    let br = &b.branches[bi];
                    if br.block >= i {
                        b.branches[bi] = Branch {
                            cond: br.cond,
                            block: br.block - 1,
                            args: br.args.clone(),
                        };
                    }
                }
            }
        }

        for (_, (b, j)) in self.defs.iter_mut().enumerate() {
            match *b == i as i32 {
                true => {
                    *b = -1;
                    *j = -1;
                }
                false => match *b > i as i32 {
                    true => *b -= i as i32,
                    false => (),
                },
            }
        }
    }
}

/////
///// IR manipulation utilities
/////

#[derive(Debug)]
pub struct MutableBlockIterator<'b, I, A> {
    ir: &'b mut ExtIR<I, A>,
    ks: Vec<Var>,
    state: usize,
}

impl<'b, I, A> Iterator for MutableBlockIterator<'b, I, A> {
    type Item = (Var, &'b mut Instruction<I, A>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.state >= self.ks.len() {
            None
        } else {
            let p = self.ir.get_instr_mut(self.ks[self.state]);
            self.state += 1;
            match p {
                None => None,
                Some((v, inst)) => {
                    // somewhat convinced this is required.
                    let ptr: *mut Instruction<I, A> = inst;
                    Some((v, unsafe { &mut *ptr }))
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ImmutableBlockIterator<'b, I, A> {
    ir: &'b ExtIR<I, A>,
    ks: Vec<Var>,
    state: usize,
}

impl<'b, I, A> Iterator for ImmutableBlockIterator<'b, I, A> {
    type Item = (Var, &'b Instruction<I, A>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.ks.len() > self.state {
            let p = self.ir.get_instr(self.ks[self.state]);
            self.state += 1;
            return p;
        }
        None
    }
}

/////
///// Verification.
/////

/*

   The verification pass checks that:

   1. Uses are always dominated by a def.
   2. Basic blocks are in extended form

*/

/////
///// Lowering.
/////

/*

   Defines the interfaces by which an AST can target
   and lower to an IR of type `T`.

*/

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

// `R` encodes the return analysis type -- it's a contract
// specified for `self.result()`.
pub trait AbstractInterpreter<IR, R> {
    type Meta;
    type LatticeElement;
    type Error;

    fn prepare(meta: Self::Meta, ir: &IR) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn step(&mut self, ir: &IR) -> Result<(), Self::Error>;

    fn result(&mut self) -> Result<R, Self::Error>;
}

/////
///// `std` features.
/////

use std::fmt;

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "%{}", self.get_id())
    }
}

impl<I> fmt::Display for Operator<I>
where
    I: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operator::Intrinsic(v) => write!(f, "{}", v),
            Operator::ModuleRef(module, name) => match module {
                None => write!(f, "@{}", name),
                Some(v) => write!(f, "{}.@{}", v, name),
            },
        }
    }
}

impl fmt::Display for Branch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "  br {} (", self.block)?;
        let l = self.args.len();
        for (ind, arg) in self.args.iter().enumerate() {
            match l - 1 == ind {
                true => write!(f, "{}", arg)?,
                _ => write!(f, "{}, ", arg)?,
            };
        }
        write!(f, ")")?;
        match self.cond {
            None => Ok(()),
            Some(v) => write!(f, " if {}", v),
        }
    }
}

impl<I, A> fmt::Display for Instruction<I, A>
where
    I: fmt::Display,
    A: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.op)?;
        write!(f, "(")?;
        let l = self.args.len();
        for (ind, arg) in self.args.iter().enumerate() {
            match l - 1 == ind {
                true => write!(f, "{}", arg)?,
                _ => write!(f, "{}, ", arg)?,
            };
        }
        write!(f, ")")?;
        if !self.attrs.is_empty() {
            write!(f, " {{ ")?;
            let l = self.attrs.len();
            for (ind, attr) in self.attrs.iter().enumerate() {
                match l - 1 == ind {
                    true => write!(f, "{}", attr)?,
                    _ => write!(f, "{}, ", attr)?,
                };
            }
            write!(f, " }}")?;
        }
        Ok(())
    }
}

impl<I, A> fmt::Display for ExtIR<I, A>
where
    I: fmt::Display,
    A: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for ind in 0..self.blocks.len() {
            write!(f, "{}: ", ind)?;
            let b = &self.blocks[ind];
            let bargs = &b.args;
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
            for (v, instr) in self.block_iter(ind) {
                writeln!(f, "  {} = {}", v, instr)?;
            }
            for br in &self.blocks[ind].branches {
                writeln!(f, "{}", br)?;
            }
        }
        Ok(())
    }
}
