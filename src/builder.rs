/*

   This file is part of `abstraps`. License is MIT.

   The builder design in this module
   supports code generation to the `abstraps` IR.

   The interfaces provided below allow customized code generation
   for user-defined intrinsics and lowering.

*/

use crate::ir::{ExtIR, Instruction, Lowering, Var};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug)]
pub enum BuilderError {
    BuilderCreationFailure,
    Caseless,
}

pub struct ExtIRBuilder<I, A> {
    block_ptr: usize,
    varmap: BTreeMap<String, Var>,
    ir: ExtIR<I, A>,
}

impl<I, A> ExtIRBuilder<I, A> {
    fn push_instr(&mut self, instr: Instruction<I, A>) -> Var {
        self.ir.push_instr(self.block_ptr, instr)
    }

    fn push_branch(&mut self, cond: Option<Var>, block: usize, args: Vec<Var>) {
        self.ir.push_branch(cond, self.block_ptr, block, args)
    }

    fn push_arg(&mut self) -> Var {
        self.ir.push_arg(self.block_ptr)
    }

    fn push_block(&mut self) -> usize {
        self.ir.push_block()
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

    fn prepare_builder(&self) -> Result<Self::IRBuilder, Self::Error> {
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
        let mut b = Lowering::<ExtIR<I, A>>::prepare_builder(self)?;
        Lowering::<ExtIR<I, A>>::build(self, &mut b)?;
        Ok(b.ir)
    }
}
