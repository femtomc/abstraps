use crate::core::graph::Graph;
use crate::core::ir::{BasicBlock, Operation, Var};
use crate::core::ssacfg::SSACFG;
use color_eyre::{eyre::bail, Report};

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
