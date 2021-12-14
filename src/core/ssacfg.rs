use crate::core::ir::{BasicBlock, Operation, Var};
use alloc::string::String;
use alloc::vec::Vec;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub struct SSACFG {
    defs: Vec<(i32, i32)>,
    blocks: Vec<BasicBlock>,
}

impl Default for SSACFG {
    fn default() -> SSACFG {
        SSACFG {
            defs: Vec::new(),
            blocks: Vec::new(),
        }
    }
}

impl SSACFG {
    pub fn get_operands(&self) -> &[Var] {
        self.blocks[0].get_operands()
    }

    pub fn get_block_operands(&self, blk: usize) -> &[Var] {
        self.blocks[blk].get_operands()
    }

    pub fn push_arg(&mut self, blk: usize) -> Var {
        let arg = Var::new(self.defs.len());
        self.defs.push((blk as i32, -1));
        self.blocks[blk].get_operands_mut().push(arg);
        arg
    }

    pub fn get_block(&mut self, ind: usize) -> &mut BasicBlock {
        &mut self.blocks[ind]
    }

    pub fn get_blocks(&self) -> &[BasicBlock] {
        &self.blocks
    }

    pub fn push_block(&mut self, blk: BasicBlock) -> usize {
        self.blocks.push(blk);
        self.blocks.len() - 1
    }

    /// Get an immutable reference to a "line" of the IR.
    /// The IR is indexed with `id` (a `Var` instance).
    /// This returns `(Var, &Operation)`.
    pub fn get_op(&self, id: Var) -> Option<(Var, &Operation)> {
        match self.get_var_blockidx(id) {
            None => None,
            Some((b, i)) => {
                let bb = &self.blocks[b];
                let inst = &bb.get_ops()[i as usize];
                Some((id, inst))
            }
        }
    }

    /// Push an operation onto the IR at block index `blk`.
    /// Returns a new `Var` reference to that operation.
    pub fn push_op(&mut self, blk: usize, v: Operation) -> Var {
        let arg = Var::new(self.defs.len());
        let len = self.blocks[blk].get_ops().len();
        let bb = &mut self.blocks[blk];
        bb.get_ops_mut().push(v);
        self.defs.push((blk as i32, len as i32));
        arg
    }

    fn get_op_mut(&mut self, id: Var) -> Option<(Var, &mut Operation)> {
        match self.get_var_blockidx(id) {
            None => None,
            Some((b, i)) => {
                let bb = &mut self.blocks[b];
                let inst = &mut bb.get_ops_mut()[i as usize];
                Some((id, inst))
            }
        }
    }

    /// Get the vector of `Var` which index into block with index `id`.
    pub fn get_block_vars(&self, id: usize) -> Vec<Var> {
        let v = self
            .defs
            .iter()
            .enumerate()
            .filter(|(_, v)| v.0 == (id as i32) && v.1 >= 0);
        let mut m = v.map(|(i, l)| (Var::new(i), l)).collect::<Vec<_>>();
        m.sort_by(|a, b| a.1.cmp(b.1));
        m.iter().map(|v| v.0).collect::<Vec<_>>()
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
            let r = Var::new(ind);
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
}
