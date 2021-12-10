/*!

   The builder design in this module
   supports code generation to the `abstraps` IR.

   The interfaces provided below allow customized code generation
   for user-defined intrinsics and lowering.

!*/

use crate::ir::{ExtIR, Instruction, Lowering, Operator, Var};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Serialize, Serializer};

#[derive(Debug)]
pub enum BuilderError {
    BuilderCreationFailure,
    Caseless,
}

#[derive(Debug)]
pub struct ExtIRBuilder<I, A> {
    block_ptr: usize,
    varmap: BTreeMap<String, Var>,
    ir: ExtIR<I, A>,
}

impl<I, A> ExtIRBuilder<I, A> {
    pub fn set_block_ptr(&mut self, ptr: usize) {
        self.block_ptr = ptr;
    }

    pub fn get_block_ptr(&self) -> usize {
        self.block_ptr
    }

    pub fn create_instr(
        &mut self,
        op: Operator<I>,
        args: Vec<Var>,
        attrs: Vec<A>,
    ) -> Instruction<I, A> {
        Instruction::new(op, args, attrs)
    }

    pub fn push_instr(&mut self, instr: Instruction<I, A>) -> Var {
        self.ir.push_instr(self.block_ptr, instr)
    }

    pub fn build_instr(&mut self, op: Operator<I>, args: Vec<Var>, attrs: Vec<A>) -> Var {
        let instr = self.create_instr(op, args, attrs);
        self.push_instr(instr)
    }

    /// Append a new `Branch` to the current block.
    pub fn push_branch(&mut self, cond: Option<Var>, block: usize, args: Vec<Var>) {
        self.ir.push_branch(cond, self.block_ptr, block, args)
    }

    /// Append a new argument `Var` to the current block.
    /// Return the `Var`.
    pub fn push_arg(&mut self) -> Var {
        self.ir.push_arg(self.block_ptr)
    }

    /// Append a block onto the builder's IR.
    /// Sets the builder's block pointer to the appended block.
    /// Returns the old block pointer.
    pub fn push_blk(&mut self) -> usize {
        let block_ptr = self.get_block_ptr();
        let new_ptr = self.ir.push_blk();
        self.set_block_ptr(new_ptr);
        block_ptr
    }

    pub fn jump_blk(&mut self, args: Vec<Var>) -> Vec<Var> {
        let block_ptr = self.get_block_ptr();
        let new_ptr = self.ir.push_blk();
        self.set_block_ptr(new_ptr);
        let block_args = args.iter().map(|_| self.push_arg()).collect();
        self.ir.push_branch(None, block_ptr, new_ptr, args);
        block_args
    }

    pub fn get_ir(&self) -> &ExtIR<I, A> {
        &self.ir
    }

    /// Returns the IR (and ownership of the IR).
    pub fn dump(self) -> ExtIR<I, A> {
        self.ir
    }
}

impl<I, A> Default for ExtIRBuilder<I, A> {
    fn default() -> ExtIRBuilder<I, A> {
        ExtIRBuilder {
            block_ptr: 0,
            varmap: BTreeMap::new(),
            ir: ExtIR::<I, A>::default(),
        }
    }
}

/// This is a focused trait which prevents implementers
/// from dealing with builder preparation and construction.
///
/// Implementing this trait for a type `T`
/// provides access to the builder lowering facilities.
pub trait ExtIRCodegen<I, A>
where
    Self: Sized,
{
    fn codegen(b: &mut ExtIRBuilder<I, A>, expr: &Self) -> Result<Var, BuilderError>;
}

impl<I, A, T> Lowering<ExtIR<I, A>> for T
where
    T: ExtIRCodegen<I, A>,
{
    type IRBuilder = ExtIRBuilder<I, A>;
    type Error = BuilderError;

    fn prepare(&self) -> Result<Self::IRBuilder, Self::Error> {
        let ir = ExtIR::<I, A>::default();
        Ok(ExtIRBuilder {
            block_ptr: 0,
            varmap: BTreeMap::new(),
            ir,
        })
    }

    fn build(&self, b: &mut Self::IRBuilder) -> Result<(), Self::Error> {
        match ExtIRCodegen::<I, A>::codegen(b, self) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn lower(&self) -> Result<ExtIR<I, A>, Self::Error> {
        let mut b = Lowering::<ExtIR<I, A>>::prepare(self)?;
        Lowering::<ExtIR<I, A>>::build(self, &mut b)?;
        Ok(b.ir)
    }
}

/////
///// Serialization.
/////

impl<I, A> Serialize for ExtIRBuilder<I, A>
where
    I: Serialize,
    A: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.ir.serialize(serializer)
    }
}

/////
///// `std` features.
/////

use std::fmt;

impl<I, A> fmt::Display for ExtIRBuilder<I, A>
where
    I: fmt::Display,
    A: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.ir)
    }
}