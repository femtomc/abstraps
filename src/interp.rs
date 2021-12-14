/*!

   An interpreter which implements lattice interpretation
   via forward propagation.

   Exposes the interface `Propagation` for users to
   customize the interpretation process.

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

use crate::ir::builder::OperationBuilder;
use crate::ir::core::{AbstractInterpreter, Operation, Var};
use alloc::collections::vec_deque::VecDeque;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/////
///// Interpreter
/////

#[derive(Debug, PartialEq)]
pub enum InterpreterError<E> {
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
    LatticeError(E),
    Caseless,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Meta<V> {
    module: Option<String>,
    name: String,
    vs: Vec<V>,
}

impl<V> Meta<V> {
    pub fn new(name: String, vs: Vec<V>) -> Meta<V> {
        Meta {
            module: None,
            name,
            vs,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_lattice_values(&self) -> &[V] {
        &self.vs
    }
}

#[derive(Debug)]
pub enum InterpreterState<V, E> {
    Inactive,
    Active,
    Waiting(Meta<V>),
    Error(InterpreterError<E>),
    Finished,
}

// This is a block interpretation frame.
// It contains an index pointer to the block.
// As well as a local interpretation environment,
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

    fn insert(&mut self, k: &Var, v: V) {
        self.block_env.insert(*k, v);
    }
}

/// This is the packaged up form of analysis
/// which the interpreter returns after working
/// on a particular operation.
#[derive(Debug)]
pub struct InterpreterFrame<C, I, A, V> {
    vs: Vec<V>,
    analysis: Option<C>,
    trace: Option<Operation<I, A>>,
}

impl<C, I, A, V> InterpreterFrame<C, I, A, V>
where
    V: Clone,
{
    pub fn get_ret(&self) -> Option<V> {
        self.vs.last().cloned()
    }
}

/// The `Interpreter` schema structure here represents one design
/// idea for "forward propagation" abstract interpretation.
///
/// Here, the `Interpreter` is parametrized by:
/// 0. `C` - any analysis state (for example, if the interpreter
/// is used to compute dependency flow analysis information).
/// 1. `I` - the IR intrinsics, user-defined.
/// 2. `A` - the IR attributes (connected to `Operation` instances, also user-defined).
/// 3. `V` - the type of lattice elements assignable to SSA variables.
/// 4. `E` - the error type associated with the process of interpretation.
/// 5. `G` - any global state which can be used to communicate to higher-level interpretation
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
/// 3. Does it record an IR trace of interpretation (here, `trace: Option<OperationBuilder<I, A>>`).
#[derive(Debug)]
pub struct Interpreter<C, I, A, V, E, G> {
    meta: Meta<V>,
    state: InterpreterState<V, E>,
    active: BlockFrame<V>,
    block_queue: VecDeque<BlockFrame<V>>,
    env: BTreeMap<Var, V>,
    analysis: Option<C>,
    trace: Option<OperationBuilder<I, A>>,
    global: Option<G>,
}

impl<C, I, A, V, E, G> Interpreter<C, I, A, V, E, G>
where
    V: Clone + LatticeJoin<E>,
{
    pub fn set_analysis(&mut self, v: Option<C>) {
        self.analysis = v;
    }

    pub fn get(&self, v: &Var) -> Option<V> {
        self.active.block_env.get(v).cloned()
    }

    pub fn merge_insert(&mut self, k: &Var, v: V) -> Result<(), InterpreterError<E>> {
        self.env.insert(*k, v);
        Ok(())
    }

    pub fn merge(&mut self) -> Result<(), InterpreterError<E>> {
        for (k, v) in &self.active.block_env {
            match self.get(k) {
                None => self.env.insert(*k, v.clone()),
                Some(t) => match t.join(v) {
                    Err(e) => return Err(InterpreterError::LatticeError(e)),
                    Ok(merged) => self.env.insert(*k, merged),
                },
            };
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
    fn propagate(&mut self, v: Var, instr: &Operation<I, A>) -> Result<V, E>;
}

/// The `BranchPrepare` trait customizes how the interpreter deals with
/// branching in the IR.
pub trait BranchPrepare<I, A, E> {
    fn prepare_branch(&mut self, ir: &Operation<I, A>, br: &Branch) -> Result<(), E>;
}

impl<C, I, A, V, E, G> BranchPrepare<I, A, InterpreterError<E>> for Interpreter<C, I, A, V, E, G>
where
    V: Clone + LatticeJoin<E> + std::cmp::PartialEq,
{
    // The assumptions here need to be carefully checked .
    fn prepare_branch(
        &mut self,
        ir: &Operation<I, A>,
        br: &Branch,
    ) -> Result<(), InterpreterError<E>> {
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
            (None, _) => Err(InterpreterError::FailedToPrepareInterpreter),
            (Some(v1), None) => {
                let mut frame = BlockFrame {
                    block_ptr: br.get_block(),
                    block_env: BTreeMap::new(),
                    lines: VecDeque::from(ir.get_vars_in_block(br.get_block())),
                };
                for (v, t) in ir.get_block_args(br.get_block()).iter().zip(v1.iter()) {
                    self.merge_insert(v, t.clone())?;
                    frame.insert(v, t.clone());
                }
                self.block_queue.push_back(frame);
                Ok(())
            }
            (Some(v1), Some(v2)) => {
                if v1 == v2 {
                    Ok(())
                } else {
                    let joined = v1
                        .iter()
                        .zip(v2.iter())
                        .map(|(a, b)| a.join(b))
                        .collect::<Result<Vec<_>, _>>();
                    match joined {
                        Err(e) => Err(InterpreterError::LatticeError(e)),
                        Ok(v3) => {
                            let mut frame = BlockFrame {
                                block_ptr: br.get_block(),
                                block_env: BTreeMap::new(),
                                lines: VecDeque::from(ir.get_vars_in_block(br.get_block())),
                            };
                            for (v, t) in ir.get_block_args(br.get_block()).iter().zip(v3.iter()) {
                                self.merge_insert(v, t.clone())?;
                                frame.insert(v, t.clone());
                            }
                            self.block_queue.push_back(frame);
                            Ok(())
                        }
                    }
                }
            }
        }
    }
}

/// The `LatticeJoin` trait provides a mechanism for converging
/// `BlockFrame` results to be joined together.
///
/// The interpreter will naturally encountered "join points"
/// in natural loops - this trait provides customization for how
/// joining occurs on lattices with lattice type `V`.
pub trait LatticeJoin<E>
where
    Self: Sized,
{
    fn join(&self, other: &Self) -> Result<Self, E>;
}

/// The `Communication` trait provides a mechanism for interpreters
/// to synchronize across some global state `G`
/// (as part of higher-level inference processes).
pub trait Communication<M, R> {
    fn ask(&self, msg: &M) -> Option<R>;
}

impl<C, I, A, M, R, V, E, G> Communication<M, R> for Interpreter<C, I, A, V, E, G>
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

impl<C, I, A, V, E, G> AbstractInterpreter<Operation<I, A>, InterpreterFrame<C, I, A, V>>
    for Interpreter<C, I, A, V, E, G>
where
    V: Clone + LatticeJoin<E> + std::cmp::PartialEq,
    G: Communication<Meta<V>, V>,
    Self: Propagation<I, A, V, E>,
{
    type LatticeElement = V;
    type Error = InterpreterError<E>;
    type Meta = Meta<V>;

    fn prepare(
        meta: Self::Meta,
        ir: &Operation<I, A>,
    ) -> Result<Interpreter<C, I, A, V, E, G>, Self::Error> {
        if ir.get_args().len() != meta.get_lattice_values().len() {
            return Err(InterpreterError::FailedToPrepareInterpreter);
        }
        let mut initial_env: BTreeMap<Var, V> = BTreeMap::new();
        for (t, v) in meta.get_lattice_values().iter().zip(ir.get_args()) {
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
            state: InterpreterState::Active,
            env: BTreeMap::<Var, V>::new(),
            analysis: None,
            trace: None,
            global: None,
        })
    }

    fn step(&mut self, ir: &Operation<I, A>) -> Result<(), Self::Error> {
        match &self.state {
            InterpreterState::Waiting(tsig) => {
                // This should never panic.
                let v = self.active.lines.pop_front().unwrap();
                match &self.global {
                    None => (),
                    Some(rr) => match rr.ask(tsig) {
                        None => (),
                        Some(t) => {
                            self.active.block_env.insert(v, t);
                            self.state = InterpreterState::Active;
                        }
                    },
                }
            }

            InterpreterState::Active => {
                let v = self.active.lines.pop_front();
                match v {
                    None => {
                        self.merge()?;
                        for br in ir.get_branches(self.active.block_ptr) {
                            self.prepare_branch(ir, br)?;
                            if !br.is_conditional() {
                                break;
                            }
                        }
                        match self.block_queue.pop_front() {
                            None => self.state = InterpreterState::Finished,
                            Some(blk) => {
                                self.active = blk;
                                self.state = InterpreterState::Active;
                            }
                        }
                    }

                    Some(el) => match ir.get_instr(el) {
                        None => {
                            self.state =
                                InterpreterState::Error(InterpreterError::FailedToLookupVarInEnv)
                        }
                        Some((v, instr)) => match instr.get_op() {
                            Operator::Intrinsic(_intr) => match self.propagate(v, instr) {
                                Ok(t) => {
                                    self.active.block_env.insert(v, t);
                                    self.state = InterpreterState::Active;
                                }
                                Err(e) => {
                                    self.state =
                                        InterpreterState::Error(InterpreterError::LatticeError(e))
                                }
                            },
                            Operator::ModuleRef(module, n) => {
                                match instr
                                    .get_args()
                                    .iter()
                                    .map(|x| self.active.get(x))
                                    .collect::<Option<Vec<_>>>()
                                {
                                    None => {
                                        self.state = InterpreterState::Error(
                                            InterpreterError::NotAllArgumentsResolvedInCall,
                                        )
                                    }
                                    Some(vs) => {
                                        self.active.lines.push_front(v);
                                        self.state = InterpreterState::Waiting(Meta {
                                            module: module.clone(),
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

    /// `finish` the interpreter -- producing an `InterpreterFrame` which
    /// contains analysis, a potential IR trace (if the interpreter is
    /// used for partial evaluation), and a lattice map.
    ///
    /// Resets the `trace` and `analysis` fields of the interpreter
    /// to `None`.
    ///
    /// Calling this function will produce an error value
    /// if the interpreter does not have
    /// `self.state == InterpreterState::Finished`.
    fn finish(&mut self) -> Result<InterpreterFrame<C, I, A, V>, Self::Error> {
        match self.state {
            InterpreterState::Finished => {
                let mut env = self.env.iter().collect::<Vec<_>>();
                env.sort_by(|a, b| a.0.get_id().partial_cmp(&b.0.get_id()).unwrap());
                let vs = env.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
                let trace = self.trace.take().map(|v| v.dump());
                let analysis = self.analysis.take();
                let frame = InterpreterFrame {
                    vs,
                    analysis,
                    trace,
                };
                Ok(frame)
            }
            _ => Err(InterpreterError::TriedToFinishWhenInterpreterNotFinished),
        }
    }
}

impl<C, I, A, V, E, G> Interpreter<C, I, A, V, E, G>
where
    C: Clone,
    I: Clone,
    A: Clone,
    V: Clone + LatticeJoin<E>,
{
    /// This function explicitly allocates to produce an intermediate
    /// (i.e. in the middle of an interpretation process) result frame.
    ///
    /// It _does not_ reset the internal state of the interpreter.
    /// It can be used to safely observe the interpreter during
    /// non-finished states.
    pub fn get_intermediate_frame(
        &self,
    ) -> Result<InterpreterFrame<C, I, A, V>, InterpreterError<E>> {
        let mut env = self.env.iter().collect::<Vec<_>>();
        env.sort_by(|a, b| a.0.get_id().partial_cmp(&b.0.get_id()).unwrap());
        let vs = env.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
        let trace = self.trace.as_ref().map(|v| v.get_ir().clone());
        let analysis = self.analysis.clone();
        let frame = InterpreterFrame {
            vs,
            analysis,
            trace,
        };
        Ok(frame)
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

impl<C, I, A, V> fmt::Display for InterpreterFrame<C, I, A, V>
where
    C: fmt::Display,
    I: fmt::Display,
    A: fmt::Display,
    V: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Frame:")?;
        for (ind, t) in self.vs.iter().enumerate() {
            write!(indented(f), "%{} :: {}\n", ind, t)?;
        }
        match &self.trace {
            None => Ok(()),
            Some(tr) => write!(f, "{}", tr),
        }?;
        match &self.analysis {
            None => Ok(()),
            Some(c) => write!(f, "{}", c),
        }?;
        Ok(())
    }
}
