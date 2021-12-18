use crate::core::builder::OperationBuilder;
use crate::core::ir::{Operation, Var};
use anyhow;
use downcast_rs::{impl_downcast, Downcast};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, PartialEq)]
pub enum InterpreterError {
    FailedToPrepareInterpreter,
    FailedToResolve,
    FailedToUnify,
    FailedToPropagate,
    MergeFailure,
    NoPropagationRuleForIntrinsic,
    FailedToLookupVarInEnv,
    FailedToLookupVarInIR,
    NotAllArgumentsResolvedInCall,
    TriedToFinishWhenInterpreterNotFinished,
    Caseless,
}

#[derive(Debug)]
pub enum InterpreterState {
    Inactive,
    Active,
    Error(InterpreterError),
    Finished,
}

// This is a block interpretation frame.
// It contains an index pointer to the block.
// As well as a local interpretation environment,
// and the vector of instruction indices
// which need to be inferred.
#[derive(Debug)]
pub struct BlockFrame<L> {
    block_ptr: usize,
    block_env: HashMap<Var, L>,
    lines: VecDeque<Var>,
}

impl<L> BlockFrame<L>
where
    L: Clone,
{
    fn get(&self, v: &Var) -> Option<L> {
        self.block_env.get(v).cloned()
    }

    fn insert(&mut self, k: &Var, v: L) {
        self.block_env.insert(*k, v);
    }
}

/// This is the packaged up form of analysis
/// which the interpreter returns after working
/// on a particular operation.
#[derive(Debug)]
pub struct InterpreterFrame<L> {
    vs: Vec<L>,
    trace: Option<Operation>,
}

impl<L> InterpreterFrame<L>
where
    L: Clone,
{
    pub fn get_ret(&self) -> Option<L> {
        self.vs.last().cloned()
    }
}

#[derive(Debug)]
pub struct Interpreter<L, G> {
    state: InterpreterState,
    active: BlockFrame<L>,
    block_queue: VecDeque<BlockFrame<L>>,
    env: HashMap<Var, L>,
    trace: Option<OperationBuilder>,
    global: Option<G>,
}

pub trait LatticeSemantics<L> {
    fn propagate(&self, v: Vec<L>) -> anyhow::Result<L>;
}

pub trait LatticeJoin {
    fn join(&self, other: &Self) -> Self;
}

pub trait LatticeConvert<L> {
    fn convert(&self) -> L;
}

/// The `Communication` trait provides a mechanism for interpreters
/// to synchronize across some global state `G`
/// (as part of higher-level inference processes).
pub trait Communication<M, R> {
    fn ask(&self, msg: &M) -> Option<R>;
}

impl<M, L, G> Communication<M, L> for Interpreter<L, G>
where
    G: Communication<M, L>,
{
    fn ask(&self, msg: &M) -> Option<L> {
        match &self.global {
            None => None,
            Some(rr) => rr.ask(msg),
        }
    }
}

impl<L, G> Interpreter<L, G>
where
    L: Clone + LatticeJoin,
{
    pub fn get(&self, v: &Var) -> Option<L> {
        self.active.block_env.get(v).cloned()
    }

    pub fn merge_insert(&mut self, k: &Var, v: L) -> Result<(), InterpreterError> {
        self.env.insert(*k, v);
        Ok(())
    }

    pub fn merge(&mut self) -> Result<(), InterpreterError> {
        for (k, v) in &self.active.block_env {
            match self.get(k) {
                None => self.env.insert(*k, v.clone()),
                Some(t) => {
                    let merged = t.join(v);
                    self.env.insert(*k, merged)
                }
            };
        }
        Ok(())
    }
}
