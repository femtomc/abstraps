use crate::core::ir::{BasicBlock, Operation, Var};
use color_eyre::{eyre::bail, Report};

#[derive(Debug, Hash)]
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
    pub fn len(&self) -> usize {
        self.defs.len()
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
        0
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
        self.defs.push((0_i32, len as i32));
        arg
    }

    pub fn get_block(&mut self) -> &mut BasicBlock {
        &mut self.blocks[0]
    }
}

#[derive(Debug, Hash)]
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
    pub fn len(&self) -> usize {
        self.defs.len()
    }

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

/// A close copy of the equivalent concept in MLIR.
///
/// A region represents a scope controlled by the parent operation.
/// The scope itself can have various attributes applied to it
/// (in MLIR, this is via the trait system).
#[derive(Debug, Hash)]
pub enum Region {
    Directed(SSACFG),
    Undirected(Graph),
}

impl Region {
    pub fn len(&self) -> usize {
        match self {
            Region::Directed(ssacfg) => ssacfg.len(),
            Region::Undirected(graph) => graph.len(),
        }
    }

    pub fn push_arg(&mut self, ind: usize) -> Result<Var, Report> {
        match self {
            Region::Directed(ssacfg) => Ok(ssacfg.push_arg(ind)),
            Region::Undirected(_graph) => {
                bail!("Can't push argument onto `Graph` region.")
            }
        }
    }

    pub fn push_op(&mut self, blk: usize, op: Operation) -> Var {
        match self {
            Region::Directed(ssacfg) => ssacfg.push_op(blk, op),
            Region::Undirected(graph) => graph.push_op(op),
        }
    }

    pub fn get_op(&self, id: Var) -> Option<(Var, &Operation)> {
        match self {
            Region::Directed(ssacfg) => ssacfg.get_op(id),
            Region::Undirected(graph) => graph.get_op(id),
        }
    }

    pub fn push_block(&mut self, b: BasicBlock) -> Result<(), Report> {
        match self {
            Region::Directed(ssacfg) => {
                ssacfg.push_block(b);
                Ok(())
            }
            Region::Undirected(graph) => {
                if graph.has_block() {
                    bail!("Can't push block onto `Graph` region which already has a block.")
                } else {
                    graph.push_block(b);
                    Ok(())
                }
            }
        }
    }

    pub fn get_block(&mut self, ind: usize) -> &mut BasicBlock {
        match self {
            Region::Directed(ssacfg) => ssacfg.get_block(ind),
            Region::Undirected(graph) => graph.get_block(),
        }
    }
}

#[derive(Debug)]
pub struct ImmutableBlockIterator<'b> {
    region: &'b Region,
    ks: Vec<Var>,
    state: usize,
}

impl<'b> Iterator for ImmutableBlockIterator<'b> {
    type Item = (Var, &'b Operation);

    fn next(&mut self) -> Option<Self::Item> {
        if self.ks.len() > self.state {
            let p = self.region.get_op(self.ks[self.state]);
            self.state += 1;
            return p;
        }
        None
    }
}

impl Region {
    /// Get an immutable iterator over basic blocks.
    pub fn get_block_iter(&self, id: usize) -> ImmutableBlockIterator {
        let ks = match self {
            Region::Directed(ssacfg) => ssacfg.get_block_vars(id),
            Region::Undirected(graph) => graph.get_block_vars(),
        };
        ImmutableBlockIterator {
            region: self,
            ks,
            state: 0,
        }
    }
}
