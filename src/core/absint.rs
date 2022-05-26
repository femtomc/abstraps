use crate::core::*;
use crate::*;
use color_eyre::{eyre::bail, Report};
use std::collections::VecDeque;
use std::fmt;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use yansi::Paint;

#[derive(Debug)]
pub enum InterpreterError {}

#[derive(Debug)]
pub enum InterpreterState<L> {
    Inactive,
    Active,
    Waiting(Signature<L>),
    Error(InterpreterError),
    Finished,
}

/// This is the packaged up form of analysis
/// which the interpreter returns after working
/// on a particular operation.
#[derive(Debug)]
pub struct InterpreterFrame<L> {
    vs: Vec<Option<L>>,
    trace: Option<Operation>,
}

impl<L> Display for InterpreterFrame<L>
where
    L: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (ind, typ) in self.vs.iter().enumerate() {
            match typ {
                None => writeln!(f, "%{}", ind)?,
                Some(v) => writeln!(f, "%{} : {}", ind, Paint::magenta(format!("{}", v)))?,
            }
        }
        Ok(())
    }
}

impl<L> InterpreterFrame<L>
where
    L: Clone,
{
    pub fn get_ret(&self) -> Option<L> {
        self.vs.last().and_then(|v| v.clone())
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
    fn propagate(&self, op: &Operation, vtypes: Vec<&L>) -> Result<L, Report>;
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

    pub fn clone_frame(&self) -> Result<InterpreterFrame<L>, Report> {
        let frame = InterpreterFrame {
            vs: self.env.to_vec(),
            trace: None,
        };
        Ok(frame)
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

    // TODO: Change.
    pub fn insert(&mut self, v: Var, l: L) {
        if self.env.len() < (v.get_id() - 1) {
            self.env.push(Some(l));
        } else {
            self.env.insert(v.get_id(), Some(l));
        }
    }

    pub fn step(&mut self, op: &Operation) -> Result<(), Report> {
        for (v, o) in op.get_regions()[0].get_block_iter(self.active) {
            let intr = o.get_intrinsic();
            match intr.query_ref::<dyn LatticeSemantics<L>>() {
                None => bail!("Intrinsic fails to support lattice semantics."),
                Some(lintr) => {
                    let vtypes = self.resolve_to_lattice(o)?;
                    let ltype = lintr.propagate(o, vtypes)?;
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
pub struct Signature<L> {
    symbol: String,
    env: Vec<Option<L>>,
}

impl<L> Signature<L> {
    pub fn new(symbol: &str, env: Vec<Option<L>>) -> Signature<L> {
        Signature {
            symbol: symbol.to_string(),
            env,
        }
    }
}

impl<L> PartialEq for Signature<L>
where
    L: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.symbol == other.symbol && self.env == other.env
    }
}

impl<L> Eq for Signature<L> where L: Eq {}

impl<L> Hash for Signature<L>
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

impl<L> AnalysisKey for Signature<L>
where
    L: 'static + Clone + LatticeJoin + Display,
{
    fn to_pass(&self, _op: &Operation) -> Box<dyn AnalysisPass> {
        let pass = LatticeInterpreterPass {
            key: self.clone(),
            result: None,
        };
        Box::new(pass)
    }
}

impl<L> Display for Signature<L>
where
    L: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(", Paint::blue(&self.symbol))?;
        let l = self.env.len();
        for (ind, lval) in self.env.iter().enumerate() {
            match ind == l - 1 {
                false => match lval {
                    None => (),
                    Some(v) => write!(f, "{}", Paint::magenta(format!("{}, ", v)).bold())?,
                },
                true => match lval {
                    None => (),
                    Some(v) => write!(f, "{}", Paint::magenta(format!("{}", v)).bold())?,
                },
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

interfaces! {
    <L: 'static + LatticeJoin + Display> Signature<L>: dyn ObjectClone,
    dyn Display,
    dyn AnalysisKey where L: Clone
}

#[derive(Debug)]
pub struct LatticeInterpreterPass<L> {
    key: Signature<L>,
    result: Option<InterpreterFrame<L>>,
}

impl<L> Display for LatticeInterpreterPass<L>
where
    L: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.result {
            None => (),
            Some(v) => write!(f, "{}", v)?,
        }
        Ok(())
    }
}

impl<L> AnalysisPass for LatticeInterpreterPass<L>
where
    L: 'static + LatticeJoin + Clone + Display,
{
    fn apply(&mut self, op: &Operation) -> Result<(), Report> {
        let mut interp = Interpreter::new(op, self.key.env.to_vec());
        interp.step(op)?;
        self.result = Some(interp.clone_frame().unwrap());
        Ok(())
    }
}

interfaces! {
    <L: 'static + LatticeJoin + Display> LatticeInterpreterPass<L>: dyn Display,
    dyn AnalysisPass where L: Clone
}
