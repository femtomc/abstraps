use crate::core::ir::{BasicBlock, Operation, Var};
use alloc::string::String;
use alloc::vec::Vec;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub struct Graph {
    defs: Vec<(i32, i32)>,
    blocks: Vec<BasicBlock>,
}

impl Default for Graph {
    fn default() -> Graph {
        Graph {
            defs: Vec::new(),
            blocks: Vec::new(),
        }
    }
}

impl Graph {
    /// Get the block index and SSA index for `v: Var`.
    fn get_var_blockidx(&self, v: Var) -> Option<(usize, i32)> {
        let (b, i) = self.defs.get(v.get_id()).unwrap_or(&(-1, -1));
        if *i < 0 {
            None
        } else {
            Some((*b as usize, *i))
        }
    }

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

    pub fn has_block(&self) -> bool {
        match self.blocks.len() {
            0 => false,
            _ => true,
        }
    }

    pub fn push_block(&mut self, blk: BasicBlock) -> usize {
        self.blocks = vec![blk];
        return 0;
    }

    /// Get the vector of `Var` which index into the region block.
    pub fn get_block_vars(&self) -> Vec<Var> {
        let v = self.defs.iter().enumerate().filter(|(_, v)| v.1 >= 0);
        let mut m = v.map(|(i, l)| (Var::new(i), l)).collect::<Vec<_>>();
        m.sort_by(|a, b| a.1.cmp(b.1));
        m.iter().map(|v| v.0).collect::<Vec<_>>()
    }
}

impl Graph {
    pub fn push_op(&mut self, v: Operation) -> Var {
        let arg = Var::new(self.defs.len());
        let len = self.blocks[0].get_ops().len();
        let bb = &mut self.blocks[0];
        bb.get_ops_mut().push(v);
        self.defs.push((0 as i32, len as i32));
        arg
    }

    pub fn get_block(&mut self) -> &mut BasicBlock {
        &mut self.blocks[0]
    }
}
