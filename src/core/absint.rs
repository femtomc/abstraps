use crate::core::builder::OperationBuilder;
use crate::core::ir::{Operation, Var};
use crate::core::pass_manager::{AnalysisKey, AnalysisPass};
use color_eyre::{eyre::bail, Report};
use downcast_rs::{impl_downcast, Downcast};
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub enum InterpreterError {}

#[derive(Debug)]
pub enum InterpreterState<L> {
    Inactive,
    Active,
    Waiting(TypeKey<L>),
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
    block_env: Vec<Option<L>>,
    lines: VecDeque<Var>,
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
pub struct Interpreter<L> {
    state: InterpreterState<L>,
    active: BlockFrame<L>,
    block_queue: VecDeque<BlockFrame<L>>,
    env: Vec<Option<L>>,
    trace: Option<OperationBuilder>,
}

pub trait LatticeSemantics<L> {
    fn propagate(&self, v: Vec<L>) -> Result<L, Report>;
}

pub trait LatticeJoin {
    fn join(&self, other: &Self) -> Self;
}

pub trait LatticeConvert<L> {
    fn convert(&self) -> L;
}

impl<L> Interpreter<L>
where
    L: Clone + LatticeJoin,
{
    pub fn new(op: &Operation, env: Vec<Option<L>>) -> Interpreter<L> {
        let bf = BlockFrame {
            block_ptr: 0,
            block_env: Vec::new(),
            lines: VecDeque::new(),
        };

        let vd = VecDeque::<BlockFrame<L>>::new();
        Interpreter {
            state: InterpreterState::Active,
            active: bf,
            block_queue: vd,
            env: env,
            trace: None,
        }
    }
}

/////
///// Analysis manager interaction.
/////

#[derive(Debug, Clone)]
pub struct TypeKey<L> {
    symbol: String,
    env: Vec<Option<L>>,
}

impl<L> TypeKey<L> {
    pub fn new(symbol: &str, env: Vec<Option<L>>) -> TypeKey<L> {
        TypeKey {
            symbol: symbol.to_string(),
            env,
        }
    }
}

impl<L> PartialEq for TypeKey<L>
where
    L: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.symbol == other.symbol && self.env == other.env
    }
}

impl<L> Eq for TypeKey<L> where L: Eq {}

impl<L> Hash for TypeKey<L>
where
    L: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.symbol.hash(state);
        for lval in self.env.iter() {
            match lval {
                None => (),
                Some(v) => v.hash(state),
            };
        }
    }
}

impl<L> AnalysisKey for TypeKey<L>
where
    L: 'static + Clone + LatticeJoin,
{
    fn to_pass(&self, op: &Operation) -> Box<dyn AnalysisPass> {
        let pass = LatticeInterpreterPass {
            key: self.clone(),
            result: None,
        };
        Box::new(pass)
    }
}

impl<L> std::fmt::Display for TypeKey<L>
where
    L: std::fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(", self.symbol)?;
        for lval in self.env.iter() {
            match lval {
                None => (),
                Some(v) => write!(f, "{}, ", v)?,
            };
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct LatticeInterpreterPass<L> {
    key: TypeKey<L>,
    result: Option<InterpreterFrame<L>>,
}

impl<L> AnalysisPass for LatticeInterpreterPass<L>
where
    L: Clone + LatticeJoin,
{
    fn apply(&mut self, op: &Operation) -> Result<(), Report> {
        let interp = Interpreter::new(op, self.key.env.to_vec());
        Ok(())
    }
}
