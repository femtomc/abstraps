/*

   This file is part of `abstraps`. License is MIT.

   An interpreter for abstract interpretation via forward propagation.

   Exposes the interface `Propagation` for users to
   customize the abstract interpretation process.

   The interpreter keeps a state machine
   representation of the local process,
   as a way to coordinate with higher-scoped module
   processes (as part of high-level languages, for instance,
   where a module-level interpreter might need to coordinate multiple
   local interpreters).

   The `Communication` interface can be used in multithreaded settings
   so that local interpreters can read from global inference state
   (e.g. as provided by a module-level interpreter, c.f. above)

*/

use crate::builder::ExtIRBuilder;
use crate::ir::{AbstractInterpreter, Branch, ExtIR, Instruction, Operator, Var};
use alloc::collections::vec_deque::VecDeque;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/////
///// Interpreter
/////

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
    Caseless,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Meta<V> {
    name: String,
    vs: Vec<V>,
}

impl<V> Meta<V> {
    pub fn new(name: String, vs: Vec<V>) -> Meta<V> {
        Meta { name, vs }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_lvalues(&self) -> &[V] {
        &self.vs
    }
}

#[derive(PartialEq, Debug)]
pub enum InferenceState<V> {
    Inactive,
    Active,
    Waiting(Meta<V>),
    Error(InterpreterError),
    Finished,
}

// This is a block typing frame.
// It contains an index pointer to the block.
// As well as a local typing environment,
// and the vector of instruction indices
// which need to be inferred.
#[derive(Debug)]
pub struct BlockFrame<V> {
    block_ptr: usize,
    block_env: BTreeMap<Var, V>,
    lines: VecDeque<Var>,
}

impl<V> BlockFrame<V>
where
    V: Clone,
{
    fn get(&self, v: &Var) -> Option<V> {
        self.block_env.get(v).cloned()
    }

    fn insert(&mut self, k: &Var, v: V) -> Result<(), InterpreterError> {
        self.block_env.insert(*k, v);
        Ok(())
    }
}

// This is the packaged up form of analysis
// which the interpreter returns after working
// on a particular method specialization.
#[derive(Debug)]
pub struct Analysis<V> {
    vs: Vec<V>,
}

/// The `Interpreter` schema structure here represents one design
/// idea for "forward propagation" abstract interpretation.
///
/// Here, the `Interpreter` is parametrized by:
/// 1. M - meta information required to prepare the `Interpreter`.
/// 2. E - the error type associated with the process of interpretation.
/// 3. V - the type of lattice elements assignable to SSA variables.
/// 4. G - any global state which can be used to communicate to higher-level interpretation
///    processes.
///
/// The general interpreter process consists of propagating across the IR
/// by constructing `BlockFrame` instances for each extended basic block,
/// entering into the frame and evaluating intrinsics line by line on the lattice `V`.
///
/// This process is user-customized by the `Propagation` API traits below.
/// Specifically, the traits customize what the interpreter does when:
/// 1. It encounters a branch (e.g. does it follow all branches, or try to pre-evaluate and skip?)
/// 2. It encounters a call to a module-scope function (e.g. how does it communicate this or try to
///    resolve this itself?)
/// 3. Does it record an IR trace of interpretation (here, `trace: Option<ExtIRBuilder<I, A>>`).
#[derive(Debug)]
pub struct Interpreter<I, A, V, G> {
    meta: Meta<V>,
    state: InferenceState<V>,
    active: BlockFrame<V>,
    block_queue: VecDeque<BlockFrame<V>>,
    env: BTreeMap<Var, V>,
    trace: Option<ExtIRBuilder<I, A>>,
    global: Option<G>,
}

impl<I, A, V, G> Interpreter<I, A, V, G>
where
    V: Clone,
{
    pub fn get(&self, v: &Var) -> Option<V> {
        self.active.block_env.get(v).cloned()
    }

    pub fn merge_insert(&mut self, k: &Var, v: V) -> Result<(), InterpreterError> {
        self.env.insert(*k, v);
        Ok(())
    }

    pub fn merge(&mut self) -> Result<(), InterpreterError> {
        for (k, v) in &self.active.block_env {
            self.env.insert(*k, v.clone());
        }
        Ok(())
    }
}

/////
///// Propagation API
/////

/// The `Propagation` trait provides a way for the interpreter to
/// evaluate the effects of an IR instruction on the lattice with lattice element type `V`.
pub trait Propagation<I, A, V, E> {
    fn propagate(&mut self, instr: &Instruction<I, A>) -> Result<V, E>;
}

/// The `BranchHandling` trait customizes how the interpreter deals with
/// branching in the IR.
pub trait BranchHandling<I, A> {
    fn prepare_branch(&mut self, ir: &ExtIR<I, A>, br: &Branch) -> Result<(), InterpreterError>;
}

impl<I, A, V, G> BranchHandling<I, A> for Interpreter<I, A, V, G>
where
    V: Clone + std::cmp::PartialEq,
{
    // The assumptions here need to be carefully checked .
    fn prepare_branch(&mut self, ir: &ExtIR<I, A>, br: &Branch) -> Result<(), InterpreterError> {
        let block_idx = br.get_block();
        let brts = br
            .get_args()
            .iter()
            .map(|v| self.get(v))
            .collect::<Option<Vec<_>>>();
        let blkts = ir
            .get_block_args(block_idx)
            .iter()
            .map(|v| self.get(v))
            .collect::<Option<Vec<_>>>();
        match (brts, blkts) {
            (None, _) => Err(InterpreterError::FailedToLookupVarInEnv),
            (Some(v1), None) => {
                let mut frame = BlockFrame {
                    block_ptr: br.get_block(),
                    block_env: BTreeMap::new(),
                    lines: VecDeque::from(ir.get_vars_in_block(br.get_block())),
                };
                for (v, t) in ir.get_block_args(br.get_block()).iter().zip(v1.iter()) {
                    self.merge_insert(v, t.clone())?;
                    frame.insert(v, t.clone())?;
                }
                self.block_queue.push_back(frame);
                Ok(())
            }
            (Some(v1), Some(v2)) => {
                if v1 == v2 {
                    Ok(())
                } else {
                    Err(InterpreterError::Caseless)
                }
            }
        }
    }
}

/// The `Communication` trait provides a mechanism for interpreters
/// to synchronize across some global state `G`
/// (as part of higher-level inference processes).
pub trait Communication<M, R> {
    fn ask(&self, msg: &M) -> Option<R>;
}

impl<I, A, M, R, V, G> Communication<M, R> for Interpreter<I, A, V, G>
where
    G: Communication<M, R>,
{
    fn ask(&self, msg: &M) -> Option<R> {
        match &self.global {
            None => None,
            Some(rr) => rr.ask(msg),
        }
    }
}

impl<I, A, G, V> AbstractInterpreter<ExtIR<I, A>, Analysis<V>> for Interpreter<I, A, V, G>
where
    V: Clone + std::cmp::PartialEq,
    G: Communication<Meta<V>, V>,
    Self: Propagation<I, A, V, InterpreterError>,
{
    type LatticeElement = V;
    type Error = InterpreterError;
    type Meta = Meta<V>;

    fn prepare(meta: Self::Meta, ir: &ExtIR<I, A>) -> Result<Interpreter<I, A, V, G>, Self::Error> {
        if ir.get_args().len() != meta.get_lvalues().len() {
            return Err(InterpreterError::FailedToPrepareInterpreter);
        }
        let mut initial_env: BTreeMap<Var, V> = BTreeMap::new();
        for (t, v) in meta.get_lvalues().iter().zip(ir.get_args()) {
            initial_env.insert(*v, t.clone());
        }
        let bf = BlockFrame {
            block_ptr: 0,
            block_env: initial_env,
            lines: VecDeque::from(ir.get_vars_in_block(0)),
        };
        let vd = VecDeque::<BlockFrame<V>>::new();
        Ok(Interpreter {
            meta,
            active: bf,
            block_queue: vd,
            state: InferenceState::Active,
            env: BTreeMap::<Var, V>::new(),
            trace: None,
            global: None,
        })
    }

    fn step(&mut self, ir: &ExtIR<I, A>) -> Result<(), Self::Error> {
        match &self.state {
            InferenceState::Waiting(tsig) => {
                // This should never panic.
                let v = self.active.lines.pop_front().unwrap();
                match &self.global {
                    None => (),
                    Some(rr) => match rr.ask(tsig) {
                        None => (),
                        Some(t) => {
                            self.active.block_env.insert(v, t);
                            self.state = InferenceState::Active;
                        }
                    },
                }
            }

            InferenceState::Active => {
                let v = self.active.lines.pop_front();
                match v {
                    None => {
                        self.merge()?;
                        for br in ir.get_branches(self.active.block_ptr) {
                            self.prepare_branch(ir, br)?;
                        }
                        match self.block_queue.pop_front() {
                            None => self.state = InferenceState::Finished,
                            Some(blk) => {
                                self.active = blk;
                                self.state = InferenceState::Active;
                            }
                        }
                    }

                    Some(el) => match ir.get_instr(el) {
                        None => {
                            self.state =
                                InferenceState::Error(InterpreterError::FailedToLookupVarInIR)
                        }
                        Some((v, instr)) => match instr.get_op() {
                            Operator::Intrinsic(_intr) => match self.propagate(instr) {
                                Ok(t) => {
                                    self.active.block_env.insert(v, t);
                                    self.state = InferenceState::Active;
                                }
                                Err(e) => self.state = InferenceState::Error(e),
                            },
                            Operator::ModuleRef(_, n) => {
                                match instr
                                    .get_args()
                                    .iter()
                                    .map(|x| self.active.get(x))
                                    .collect::<Option<Vec<_>>>()
                                {
                                    None => {
                                        self.state = InferenceState::Error(
                                            InterpreterError::NotAllArgumentsResolvedInCall,
                                        );
                                    }
                                    Some(vs) => {
                                        self.active.lines.push_front(v);
                                        self.state = InferenceState::Waiting(Meta {
                                            name: n.to_string(),
                                            vs,
                                        });
                                    }
                                }
                            }
                        },
                    },
                }
            }
            _ => (),
        };
        Ok(())
    }

    fn result(&mut self) -> Result<Analysis<V>, Self::Error> {
        let mut env = self.env.iter().collect::<Vec<_>>();
        env.sort_by(|a, b| a.0.get_id().partial_cmp(&b.0.get_id()).unwrap());
        let vs = env.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
        let analysis = Analysis { vs };
        Ok(analysis)
    }
}

/////
///// `std` features.
/////

use {indenter::indented, std::fmt, std::fmt::Write};

impl<V> fmt::Display for Meta<V>
where
    V: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "@{} (", self.name)?;
        let l = self.vs.len();
        for (ind, t) in self.vs.iter().enumerate() {
            if ind == l - 1 {
                write!(f, "{}", t)?;
            } else {
                write!(f, "{},", t)?;
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl<V> fmt::Display for Analysis<V>
where
    V: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Analysis:")?;
        for (ind, t) in self.vs.iter().enumerate() {
            write!(indented(f), "%{} :: {}", ind, t)?;
        }
        Ok(())
    }
}
