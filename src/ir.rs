/*

   This file is part of `abstraps`. License is MIT.

   The design of this IR is heavily influenced by
   WebAssembly, Julia's IRTools IR, Julia's IRCode IR,
   and MLIR.

   This IR uses parametrized basic blocks (in contrast to phi nodes).

   The core of the IR is the `Instruction<T, K>` template,
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

// Supports serialization to (compressed) bincode.
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/////
///// ExtIR - an IR specialized to represent function-like dataflow.
/////

#[derive(Clone, Debug)]
pub enum IRError {
    Fallback,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone, Debug)]
pub struct Var {
    pub id: usize,
}

pub fn var(id: usize) -> Var {
    Var { id }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Operator<T> {
    Intrinsic(T),
    ModuleRef(Vec<String>, String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BranchCondition {
    None,
    Some(Var),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Branch {
    cond: BranchCondition,
    block: usize,
    args: Vec<Var>,
}

impl Branch {
    fn arguments(&mut self) -> &mut Vec<Var> {
        &mut self.args
    }

    pub fn block(&self) -> usize {
        self.block
    }

    fn isconditional(&self) -> bool {
        match self.cond {
            BranchCondition::Some(_v) => true,
            BranchCondition::None => false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Instruction<T, K> {
    pub op: Operator<T>,
    pub args: Vec<Var>,
    pub attrs: Vec<K>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BasicBlock<T, K> {
    insts: Vec<Instruction<T, K>>,
    pub args: Vec<Var>,
    pub branches: Vec<Branch>,
}

impl<T, K> Default for BasicBlock<T, K> {
    fn default() -> BasicBlock<T, K> {
        BasicBlock {
            insts: Vec::new(),
            args: Vec::new(),
            branches: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IRLInfo {
    file: String,
    module: String,
    line: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtIR<T, K> {
    defs: Vec<(i32, i32)>,
    pub blocks: Vec<BasicBlock<T, K>>,
    lines: Vec<IRLInfo>,
}

impl<T, K> Default for ExtIR<T, K> {
    fn default() -> ExtIR<T, K> {
        let entry = BasicBlock::default();
        ExtIR {
            defs: Vec::new(),
            blocks: vec![entry],
            lines: Vec::new(),
        }
    }
}

impl<T, K> ExtIR<T, K> {
    pub fn get_args(&self) -> Vec<Var> {
        let b = &self.blocks[0];
        b.args.to_vec()
    }

    pub fn push_arg(&mut self, blk: usize) -> Var {
        let arg = var(self.defs.len());
        self.defs.push((blk as i32, -1));
        self.blocks[blk].args.push(arg);
        arg
    }

    pub fn push_block(&mut self) -> usize {
        let blk = BasicBlock::default();
        self.blocks.push(blk);

        self.blocks.len() - 1
    }

    pub fn get_branches(&self, blk: usize) -> &Vec<Branch> {
        &self.blocks[blk].branches
    }

    pub fn get_branches_mut(&mut self, blk: usize) -> &mut Vec<Branch> {
        &mut self.blocks[blk].branches
    }

    pub fn push_branch(&mut self, cond: BranchCondition, from: usize, to: usize, args: Vec<Var>) {
        let brs = self.get_branches_mut(from);
        match cond {
            BranchCondition::None => brs.retain(|x| x.isconditional()),
            BranchCondition::Some(_v) => (),
        }
        brs.push(Branch {
            cond,
            block: to,
            args,
        });
    }

    pub fn get_instr(&self, id: Var) -> Option<(Var, &Instruction<T, K>)> {
        match self.get_var_blockidx(id) {
            None => None,
            Some((b, i)) => {
                let bb = &self.blocks[b];
                let inst = &bb.insts[i as usize];
                Some((id, inst))
            }
        }
    }

    pub fn push_instr(&mut self, blk: usize, v: Instruction<T, K>) -> Var {
        let arg = var(self.defs.len());
        let len = self.blocks[blk].insts.len();
        let bb = &mut self.blocks[blk];
        bb.insts.push(v);
        self.defs.push((blk as i32, len as i32));
        arg
    }

    fn get_instr_mut(&mut self, id: Var) -> Option<(Var, &mut Instruction<T, K>)> {
        match self.get_var_blockidx(id) {
            None => None,
            Some((b, i)) => {
                let bb = &mut self.blocks[b];
                let inst = &mut bb.insts[i as usize];
                Some((id, inst))
            }
        }
    }

    pub fn get_block_vars(&self, id: usize) -> Vec<Var> {
        let v = self
            .defs
            .iter()
            .enumerate()
            .filter(|(_, v)| v.0 == (id as i32) && v.1 >= 0);
        let mut m = v.map(|(i, l)| (var(i), l)).collect::<Vec<_>>();
        m.sort_by(|a, b| a.1.cmp(b.1));
        m.iter().map(|v| v.0).collect::<Vec<_>>()
    }

    pub fn block_iter(&self, id: usize) -> ImmutableBlockIterator<T, K> {
        let ks = self.get_block_vars(id);
        ImmutableBlockIterator {
            ir: self,
            ks,
            state: 0,
        }
    }

    pub fn block_iter_mut(&mut self, id: usize) -> MutableBlockIterator<T, K> {
        let ks = self.get_block_vars(id);
        MutableBlockIterator {
            ir: self,
            ks,
            state: 0,
        }
    }

    fn get_var_blockidx(&self, v: Var) -> Option<(usize, i32)> {
        let (b, i) = self.defs.get(v.id).unwrap_or(&(-1, -1));
        if *i < 0 {
            None
        } else {
            Some((*b as usize, *i))
        }
    }

    pub fn get_vars_in_block(&self, blockidx: usize) -> Vec<Var> {
        let mut v: Vec<Var> = Vec::new();
        for ind in 0..self.defs.len() {
            let r = Var { id: ind };
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
                            cond: br.cond.clone(),
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

#[cfg(test)]
mod extir_tests {
    use super::*;

    #[derive(Debug)]
    enum FakeIntrinsic {
        Null,
    }

    #[derive(Debug)]
    enum FakeAttribute {
        Null,
    }

    #[test]
    fn test_construction() {
        let _ir = ExtIR::<FakeIntrinsic, FakeAttribute>::default();
    }
}

/////
///// IR manipulation utilities
/////

#[derive(Debug)]
pub struct MutableBlockIterator<'b, T, K> {
    ir: &'b mut ExtIR<T, K>,
    ks: Vec<Var>,
    state: usize,
}

impl<'b, T, K> Iterator for MutableBlockIterator<'b, T, K> {
    type Item = (Var, &'b mut Instruction<T, K>);

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
                    let ptr: *mut Instruction<T, K> = inst;
                    Some((v, unsafe { &mut *ptr }))
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ImmutableBlockIterator<'b, T, K> {
    ir: &'b ExtIR<T, K>,
    ks: Vec<Var>,
    state: usize,
}

impl<'b, T, K> Iterator for ImmutableBlockIterator<'b, T, K> {
    type Item = (Var, &'b Instruction<T, K>);

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
///// Lowering.
/////

/*

   Defines the interfaces by which an AST can target
   and lower to an IR of type `T`.

*/

pub trait Lowering<T> {
    type IRBuilder;
    type Error;
    fn prepare_builder(&self) -> Result<Self::IRBuilder, Self::Error>;
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
    type Error;
    type LatticeElement;

    // This is basically "state required to prepare an
    // interpreter, before `step`".
    //
    // For typing, this might be type annotations.
    type Meta;

    fn prepare(meta: Self::Meta, ir: &IR) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn step(&mut self, ir: &IR) -> Result<(), Self::Error>;

    fn result(&mut self) -> Result<R, Self::Error>;
}

/////
///// `std` features.
/////

#[cfg(feature = "std")]
use std::fmt;

#[cfg(feature = "std")]
impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "%{}", self.id)
    }
}

#[cfg(feature = "std")]
impl<T> fmt::Display for Operator<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operator::Intrinsic(v) => write!(f, "{}", v),
            Operator::ModuleRef(module, name) => {
                for k in module.iter() {
                    write!(f, "{}.", k)?;
                }
                write!(f, "@{}", name)
            }
        }
    }
}

#[cfg(feature = "std")]
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
            BranchCondition::None => Ok(()),
            BranchCondition::Some(v) => write!(f, " if {}", v),
        }
    }
}

#[cfg(feature = "std")]
impl<T, K> fmt::Display for Instruction<T, K>
where
    T: fmt::Display,
    K: fmt::Display,
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

#[cfg(feature = "std")]
impl<T, K> fmt::Display for ExtIR<T, K>
where
    T: fmt::Display,
    K: fmt::Display,
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
