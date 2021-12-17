/*!

   The builder design in this module
   supports code generation to the `abstraps` IR.

   The interfaces provided below allow customized code generation
   for user-defined intrinsics and lowering.

*/

use crate::core::diagnostics::LocationInfo;
use crate::core::ir::{
    Attribute, BasicBlock, Intrinsic, IntrinsicTrait, Operation, SupportsVerification, Var,
};
use crate::core::region::Region;
use alloc::string::String;
use alloc::vec::Vec;
use anyhow::{bail, Result};
use std::collections::HashMap;

#[derive(Debug)]
pub enum BuilderError {
    BuilderCreationFailure,
    Caseless,
}

#[derive(Debug)]
pub struct OperationBuilder {
    latest: Vec<Var>,
    cursor: (usize, usize),
    location: Option<LocationInfo>,
    intrinsic: Box<dyn Intrinsic>,
    operands: Vec<Var>,
    attributes: HashMap<String, Box<dyn Attribute>>,
    regions: Vec<Region>,
    successors: Vec<BasicBlock>,
}

impl SupportsVerification for OperationBuilder {
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

impl OperationBuilder {
    pub fn default(intr: Box<dyn Intrinsic>, loc: Option<LocationInfo>) -> OperationBuilder {
        OperationBuilder {
            latest: Vec::new(),
            cursor: (0, 0),
            intrinsic: intr,
            location: loc,
            operands: Vec::new(),
            attributes: HashMap::new(),
            regions: Vec::new(),
            successors: Vec::new(),
        }
    }

    pub fn get_latest(&self) -> Vec<Var> {
        self.latest.to_vec()
    }

    pub fn get_intrinsic(&self) -> &Box<dyn Intrinsic> {
        &self.intrinsic
    }

    pub fn get_location(&self) -> &Option<LocationInfo> {
        &self.location
    }

    pub fn push_operand(&mut self, arg: Var) {
        self.operands.push(arg);
    }

    pub fn set_operands(&mut self, args: Vec<Var>) {
        self.operands = args;
    }

    pub fn get_operands(&self) -> Vec<Var> {
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

    pub fn insert_attr(&mut self, k: &str, attr: Box<dyn Attribute>) {
        self.attributes.insert(k.to_string(), attr);
    }

    pub fn get_attributes(&self) -> &HashMap<String, Box<dyn Attribute>> {
        &self.attributes
    }

    pub fn get_attributes_mut(&mut self) -> &mut HashMap<String, Box<dyn Attribute>> {
        &mut self.attributes
    }

    pub fn push_region(&mut self, r: Region) {
        self.regions.push(r);
        self.cursor = (self.cursor.0 + 1, self.cursor.1)
    }

    pub fn get_region(&mut self) -> &mut Region {
        let reg = self.cursor.0 - 1;
        &mut self.regions[reg]
    }

    pub fn get_regions(&self) -> &[Region] {
        &self.regions
    }

    pub fn push_block(&mut self, b: BasicBlock) -> Result<()> {
        let r = self.get_region();
        r.push_block(b)?;
        self.cursor = (self.cursor.0, self.cursor.1 + 1);
        Ok(())
    }

    pub fn get_block(&mut self) -> &mut BasicBlock {
        let cursor = self.cursor;
        let blk = cursor.1 - 1;
        let b = self.get_region().get_block(blk);
        b
    }

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

    pub fn push(&mut self, v: OperationBuilder) -> Result<Var> {
        let op = v.finish()?;
        Ok(self.push_op(op))
    }

    pub fn push_op(&mut self, v: Operation) -> Var {
        let ret = {
            let blk = self.get_cursor().1 - 1;
            let r = self.get_region();
            r.push_op(blk, v)
        };
        ret
    }
}

impl OperationBuilder {
    pub fn finish(self) -> Result<Operation> {
        Ok(Operation::new(
            self.location,
            self.intrinsic,
            self.operands,
            self.attributes,
            self.regions,
            self.successors,
        ))
    }
}
