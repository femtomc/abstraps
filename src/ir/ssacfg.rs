use crate::ir::core::{BasicBlock, IRLInfo, ImmutableBlockIterator, Operation, Region, Var};
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct SSACFG<I, A> {
    defs: Vec<(i32, i32)>,
    lines: Vec<Option<IRLInfo>>,
    blocks: Vec<BasicBlock<I, A>>,
}

impl<I, A> Default for SSACFG<I, A> {
    fn default() -> SSACFG<I, A> {
        SSACFG {
            defs: Vec::new(),
            blocks: Vec::new(),
            lines: Vec::new(),
        }
    }
}

impl<I1, A1> SSACFG<I1, A1> {
    pub fn pass<R>(&self, f: &dyn Fn(&Operation<I1, A1>) -> R, accum: &dyn Fn(Vec<R>) -> R) -> R {
        let blocks = self
            .blocks
            .iter()
            .map(|blk| blk.pass(f, accum))
            .collect::<Vec<_>>();
        accum(blocks)
    }

    pub fn bifmap<I2, A2>(
        mut self,
        fintr: &dyn Fn(I1) -> I2,
        fattr: &dyn Fn(A1) -> A2,
    ) -> SSACFG<I2, A2> {
        let blocks = self
            .blocks
            .into_iter()
            .map(|blk| blk.bifmap(fintr, fattr))
            .collect::<Vec<_>>();
        SSACFG {
            defs: self.defs,
            lines: self.lines,
            blocks: blocks,
        }
    }
}

impl<I, A> SSACFG<I, A> {
    pub fn get_args(&self) -> &[Var] {
        self.blocks[0].get_args()
    }

    pub fn get_block_args(&self, blk: usize) -> &[Var] {
        self.blocks[blk].get_args()
    }

    pub fn push_arg(&mut self, blk: usize) -> Var {
        let arg = Var::new(self.defs.len());
        self.defs.push((blk as i32, -1));
        self.blocks[blk].get_args_mut().push(arg);
        arg
    }

    pub fn get_block(&mut self, ind: usize) -> &mut BasicBlock<I, A> {
        &mut self.blocks[ind]
    }

    pub fn get_blocks(&self) -> &[BasicBlock<I, A>] {
        &self.blocks
    }

    pub fn push_block(&mut self, blk: BasicBlock<I, A>) -> usize {
        self.blocks.push(blk);
        self.blocks.len() - 1
    }

    /// Get an immutable reference to a "line" of the IR.
    /// The IR is indexed with `id` (a `Var` instance).
    /// This returns `(Var, &Operation<I, A>)`.
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

    /// Push an operation onto the IR at block index `blk`.
    /// Returns a new `Var` reference to that operation.
    pub fn push_op(&mut self, blk: usize, v: Operation<I, A>) -> Var {
        let arg = Var::new(self.defs.len());
        let len = self.blocks[blk].get_ops().len();
        let bb = &mut self.blocks[blk];
        bb.get_ops_mut().push(v);
        self.defs.push((blk as i32, len as i32));
        arg
    }

    fn get_op_mut(&mut self, id: Var) -> Option<(Var, &mut Operation<I, A>)> {
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
