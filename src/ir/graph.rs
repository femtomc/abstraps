use crate::ir::core::{BasicBlock, IRLInfo, Operation, Region, Var};
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct Graph<I, A> {
    defs: Vec<(i32, i32)>,
    lines: Vec<Option<IRLInfo>>,
    blocks: Vec<BasicBlock<I, A>>,
}

impl<I, A> Default for Graph<I, A> {
    fn default() -> Graph<I, A> {
        Graph {
            defs: Vec::new(),
            lines: Vec::new(),
            blocks: Vec::new(),
        }
    }
}

impl<I, A> Graph<I, A> {
    /// Get the block index and SSA index for `v: Var`.
    fn get_var_blockidx(&self, v: Var) -> Option<(usize, i32)> {
        let (b, i) = self.defs.get(v.get_id()).unwrap_or(&(-1, -1));
        if *i < 0 {
            None
        } else {
            Some((*b as usize, *i))
        }
    }

    pub fn get_op(&self, id: Var) -> Option<(Var, &Operation<I, A>)> {
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

    pub fn push_block(&mut self, blk: BasicBlock<I, A>) -> usize {
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

impl<I1, A1> Graph<I1, A1> {
    pub fn pass<R>(&self, f: &dyn Fn(&Operation<I1, A1>) -> R, accum: &dyn Fn(Vec<R>) -> R) -> R {
        let blk = &self.blocks[0];
        blk.pass(f, accum)
    }

    pub fn bifmap<I2, A2>(
        mut self,
        fintr: &dyn Fn(I1) -> I2,
        fattr: &dyn Fn(A1) -> A2,
    ) -> Graph<I2, A2> {
        let blocks = self
            .blocks
            .into_iter()
            .map(|blk| blk.bifmap(fintr, fattr))
            .collect::<Vec<_>>();
        Graph {
            defs: self.defs,
            lines: self.lines,
            blocks,
        }
    }
}

impl<I, A> Graph<I, A> {
    pub fn push_op(&mut self, v: Operation<I, A>) -> Var {
        let arg = Var::new(self.defs.len());
        let len = self.blocks[0].get_ops().len();
        let bb = &mut self.blocks[0];
        bb.get_ops_mut().push(v);
        self.defs.push((0 as i32, len as i32));
        arg
    }

    pub fn get_block(&mut self) -> &mut BasicBlock<I, A> {
        &mut self.blocks[0]
    }
}
