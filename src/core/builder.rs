use crate::core::diagnostics::LocationInfo;
use crate::core::ir::{Attribute, BasicBlock, Intrinsic, Operation, SupportsInterfaceTraits, Var};
use crate::core::region::Region;
use color_eyre::Report;
use std::collections::HashMap;

#[derive(Debug)]
pub struct OperationBuilder {
    latest: Vec<Var>,
    cursor: (usize, usize),
    location: LocationInfo,
    intrinsic: Box<dyn Intrinsic>,
    operands: Vec<Var>,
    attributes: HashMap<String, Box<dyn Attribute>>,
    regions: Vec<Region>,
    successors: Vec<usize>,
}

impl SupportsInterfaceTraits for OperationBuilder {
    fn get_intrinsic(&self) -> &Box<dyn Intrinsic> {
        &self.intrinsic
    }

    fn get_operands(&self) -> &[Var] {
        &self.operands
    }

    fn get_regions(&self) -> &[Region] {
        &self.regions
    }

    fn get_attributes(&self) -> &HashMap<String, Box<dyn Attribute>> {
        &self.attributes
    }

    fn get_attributes_mut(&mut self) -> &mut HashMap<String, Box<dyn Attribute>> {
        &mut self.attributes
    }
}

impl OperationBuilder {
    pub fn default(intr: Box<dyn Intrinsic>, loc: LocationInfo) -> OperationBuilder {
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

    pub fn get_location(&self) -> &LocationInfo {
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

    pub fn set_successors(&mut self, blks: Vec<usize>) {
        self.successors = blks;
    }

    pub fn set_cursor(&mut self, reg: usize, blk: usize) {
        self.cursor = (reg, blk);
    }

    pub fn get_cursor(&self) -> (usize, usize) {
        self.cursor
    }

    pub fn push_arg(&mut self) -> Result<Var, Report> {
        let blk = self.cursor.1 - 1;
        let r = self.get_region_mut();
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

    pub fn get_region(&self) -> &Region {
        let reg = self.cursor.0 - 1;
        &self.regions[reg]
    }

    pub fn get_region_mut(&mut self) -> &mut Region {
        let reg = self.cursor.0 - 1;
        &mut self.regions[reg]
    }

    pub fn get_regions(&self) -> &[Region] {
        &self.regions
    }

    pub fn push_block(&mut self, b: BasicBlock) -> Result<(), Report> {
        let r = self.get_region_mut();
        r.push_block(b)?;
        self.cursor = (self.cursor.0, self.cursor.1 + 1);
        Ok(())
    }

    pub fn get_block(&self) -> &BasicBlock {
        let cursor = self.cursor;
        let blk = cursor.1 - 1;
        let b = self.get_region().get_block(blk);
        b
    }

    pub fn get_block_mut(&mut self) -> &mut BasicBlock {
        let cursor = self.cursor;
        let blk = cursor.1 - 1;
        let b = self.get_region_mut().get_block_mut(blk);
        b
    }

    pub fn push(&mut self, v: OperationBuilder) -> Result<Var, Report> {
        let op = v.finish()?;
        Ok(self.push_op(op))
    }

    pub fn push_op(&mut self, v: Operation) -> Var {
        let ret = {
            let blk = self.get_cursor().1 - 1;
            let r = self.get_region_mut();
            r.push_op(blk, v)
        };
        ret
    }
}

impl OperationBuilder {
    pub fn finish(self) -> Result<Operation, Report> {
        let op = Operation::new(
            self.location,
            self.intrinsic,
            self.operands,
            self.attributes,
            self.regions,
            self.successors,
        );
        let intr = op.get_intrinsic();
        intr.verify(intr, &op)?;
        Ok(op)
    }
}
