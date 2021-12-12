/*!

   The builder design in this module
   supports code generation to the `abstraps` IR.

   The interfaces provided below allow customized code generation
   for user-defined intrinsics and lowering.

*/

use crate::ir::builtin::{Symbol, SymbolTable};
use crate::ir::core::{BasicBlock, Operation, Region, Var, Verify};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use anyhow::Result;
use serde::{Serialize, Serializer};
use std::collections::HashMap;

#[derive(Debug)]
pub enum BuilderError {
    BuilderCreationFailure,
    Caseless,
}

#[derive(Debug)]
pub struct OperationBuilder<I, A> {
    latest: Vec<Var>,
    cursor: (usize, usize),
    intrinsic: I,
    operands: Vec<Var>,
    attrs: HashMap<String, A>,
    regions: Vec<Region<I, A>>,
    successors: Vec<BasicBlock<I, A>>,
}

impl<I1, A1> OperationBuilder<I1, A1> {
    pub fn bifmap<I2, A2>(
        mut self,
        fintr: &dyn Fn(I1) -> I2,
        fattr: &dyn Fn(A1) -> A2,
    ) -> OperationBuilder<I2, A2> {
        let latest = self.latest;
        let cursor = self.cursor;
        let operands = self.operands;
        let intrinsic = fintr(self.intrinsic);
        let attrs = self
            .attrs
            .drain()
            .map(|(k, v)| (k, fattr(v)))
            .collect::<HashMap<_, _>>();
        let regions = self
            .regions
            .into_iter()
            .map(|r| r.bifmap(fintr, fattr))
            .collect::<Vec<_>>();
        let successors = self
            .successors
            .into_iter()
            .map(|blk| blk.bifmap(fintr, fattr))
            .collect::<Vec<_>>();
        OperationBuilder {
            latest,
            cursor,
            intrinsic,
            operands,
            attrs,
            regions,
            successors,
        }
    }
}

impl<I, A> OperationBuilder<I, A> {
    pub fn default(intr: I) -> OperationBuilder<I, A> {
        OperationBuilder {
            latest: Vec::new(),
            cursor: (0, 0),
            intrinsic: intr,
            operands: Vec::new(),
            attrs: HashMap::new(),
            regions: Vec::new(),
            successors: Vec::new(),
        }
    }

    pub fn get_latest(&self) -> Vec<Var> {
        self.latest.to_vec()
    }

    pub fn get_intrinsic(&self) -> I
    where
        I: Copy,
    {
        self.intrinsic
    }

    pub fn push_operand(&mut self, arg: Var) {
        self.operands.push(arg);
    }

    pub fn set_operands(&mut self, args: Vec<Var>) {
        self.operands = args;
    }

    pub fn get_operands(&self, arg: Var) -> Vec<Var> {
        self.operands.to_vec()
    }

    pub fn set_cursor(&mut self, reg: usize, blk: usize) {
        self.cursor = (reg, blk);
    }

    pub fn get_cursor(&self) -> (usize, usize) {
        self.cursor
    }

    pub fn push_arg(&mut self) -> Result<Var> {
        let blk = self.cursor.1 - 1;
        let r = self.get_region();
        match r.push_arg(blk) {
            Ok(v) => {
                if blk == 0 {
                    self.push_operand(v);
                }
                Ok(v)
            }
            Err(e) => Err(e),
        }
    }

    pub fn insert_attr(&mut self, k: &str, attr: A) {
        self.attrs.insert(k.to_string(), attr);
    }

    pub fn get_attrs(&self) -> &HashMap<String, A> {
        &self.attrs
    }

    pub fn get_attr(&self, key: &str) -> Option<&A> {
        self.attrs.get(key)
    }

    pub fn push_region(&mut self, r: Region<I, A>) {
        self.regions.push(r);
        self.cursor = (self.cursor.0 + 1, self.cursor.1)
    }

    pub fn get_region(&mut self) -> &mut Region<I, A> {
        let reg = self.cursor.0 - 1;
        &mut self.regions[reg]
    }

    pub fn get_regions(&self) -> &[Region<I, A>] {
        &self.regions
    }

    pub fn push_block(&mut self, b: BasicBlock<I, A>) -> Result<()> {
        let r = self.get_region();
        r.push_block(b)?;
        self.cursor = (self.cursor.0, self.cursor.1 + 1);
        Ok(())
    }

    pub fn get_block(&mut self) -> &mut BasicBlock<I, A> {
        let cursor = self.cursor;
        let blk = cursor.1 - 1;
        let b = self.get_region().get_block(blk);
        return b;
    }
}

/////
///// Builder setup.
/////

pub trait Setup<I, A> {
    fn new(intr: I) -> OperationBuilder<I, A>;
}

/////
///// Dialect conversion.
/////

pub trait Conversion<T>
where
    Self: Sized,
{
    fn convert(b: T) -> Self;
}

impl<I1, A1, I2, A2> Conversion<OperationBuilder<I1, A1>> for OperationBuilder<I2, A2>
where
    I2: From<I1>,
    A2: From<A1>,
{
    fn convert(b: OperationBuilder<I1, A1>) -> OperationBuilder<I2, A2> {
        let new = b.bifmap(&|x| I2::from(x), &|x| A2::from(x));
        new
    }
}

/////
///// Push conversion.
/////

pub trait Push<I2, A2, I1, A1>
where
    I2: From<I1>,
    A2: From<A2>,
{
    fn push_op(self, v: Operation<I2, A2>) -> OperationBuilder<I2, A2>;
}

impl<I2, A2, I1, A1> Push<I2, A2, I1, A1> for OperationBuilder<I1, A1>
where
    I2: From<I1>,
    A2: From<A1>,
{
    // This automatically handles dialect conversion requirements.
    // In the future, either remove -- or make fast.
    fn push_op(self, v: Operation<I2, A2>) -> OperationBuilder<I2, A2> {
        let mut b: OperationBuilder<I2, A2> = Conversion::<OperationBuilder<I1, A1>>::convert(self);
        let ret = {
            let blk = b.get_cursor().1 - 1;
            let r = b.get_region();
            r.push_op(blk, v)
        };
        b.latest = vec![ret];
        b
    }
}

/////
///// Verification.
/////

impl<I, A> OperationBuilder<I, A> {
    pub fn finish(self) -> Result<Operation<I, A>> {
        Ok(Operation::new(
            self.intrinsic,
            self.operands,
            self.attrs,
            self.regions,
            self.successors,
        ))
    }
}
