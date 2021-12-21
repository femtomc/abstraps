use crate::core::builder::OperationBuilder;
use crate::core::ir::{Operation, SupportsInterfaceTraits, Var};
use crate::core::pass_manager::AnalysisPass;
use color_eyre::{eyre::bail, Report};
use std::collections::VecDeque;
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
    active: usize,
    block_queue: VecDeque<usize>,
    env: Vec<Option<L>>,
    trace: Option<OperationBuilder>,
}

pub trait LatticeSemantics<L> {
    fn propagate(&self, v: Vec<&L>) -> Result<L, Report>;
}

pub trait LatticeJoin {
    fn join(&self, other: &Self) -> Self;
}

pub trait LatticeConvert<L> {
    fn convert(&self) -> L;
}

impl<L> Interpreter<L>
where
    L: Clone + LatticeJoin + 'static,
{
    pub fn new(_op: &Operation, env: Vec<Option<L>>) -> Interpreter<L> {
        let vd = VecDeque::<usize>::new();
        Interpreter {
            state: InterpreterState::Active,
            active: 0,
            block_queue: vd,
            env,
            trace: None,
        }
    }

    pub fn get(&self, v: Var) -> Result<&L, Report> {
        match self.env.get(v.get_id()) {
            Some(l) => match l {
                Some(l) => Ok(l),
                None => bail!(format!("No type for SSA variable {}.", v)),
            },
            None => bail!(format!("No type for SSA variable {}.", v)),
        }
    }

    pub fn resolve_to_lattice(&self, op: &Operation) -> Result<Vec<&L>, Report> {
        let operands = op.get_operands();
        operands
            .into_iter()
            .map(|v| self.get(v))
            .collect::<Result<Vec<_>, _>>()
    }

    pub fn insert(&mut self, _v: Var, _l: L) {}

    pub fn step(&mut self, op: &Operation) -> Result<(), Report> {
        for (v, o) in op.get_regions()[0].get_block_iter(self.active) {
            let intr = o.get_intrinsic();
            match intr.query_ref::<dyn LatticeSemantics<L>>() {
                None => bail!("Intrinsic fails to support lattice semantics."),
                Some(lintr) => {
                    let lvec = self.resolve_to_lattice(o)?;
                    let ltype = lintr.propagate(lvec)?;
                    self.insert(v, ltype);
                }
            }
        }
        Ok(())
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
    L: Clone + LatticeJoin + 'static,
{
    fn apply(&mut self, op: &Operation) -> Result<(), Report> {
        let mut interp = Interpreter::new(op, self.key.env.to_vec());
        interp.step(op);
        Ok(())
    }
}
