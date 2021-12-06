/*

   This file is part of `abstraps`. License is MIT.

   An interpreter for forward type propagation.

   Supports dataflow-based inference algorithms like those
   present in Crystal, or Julia

   Exposes the inference interface `Propagation` for users to customize
   the typing.

   The interpreter keeps a state machine
   representation of the local inference process,
   as a way to coordinate with higher-scoped module inference
   processes (as part of high-level languages, for instance,
   where a module-level interpreter might need to coordinate multiple
   local interpreters).

   The `Communication` interface can be used in multithreaded settings
   so that local interpreters can read from global inference state
   (e.g. as provided by a module-level interpreter, c.f. above)

*/

use crate::ir::{AbstractInterpreter, ExtIR, Instruction, Operator, Var};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};

/////
///// Interpreter
/////

#[derive(Debug, PartialEq)]
pub enum TypeError {
    FailedToResolveToType,
    FailedToUnify,
    FailedToPropagate,
    TypeMergeFailure,
    NoPropagationRuleForIntrinsic,
    FailedToLookupVarInTypeEnv,
    FailedToLookupVarInIR,
    NotAllArgumentsTypedInCall,
    Caseless,
}

// This is `Meta` for the `Interpreter`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeSignature<Ty> {
    pub name: String,
    pub ts: Vec<Ty>,
}

#[derive(PartialEq, Debug)]
pub enum InferenceState<Ty> {
    Inactive,
    Active,
    Waiting(TypeSignature<Ty>),
    Error(TypeError),
    Finished,
}

// This is a block typing frame.
// It contains an index pointer to the block.
// As well as a local typing environment,
// and the vector of instruction indices
// which need to be inferred.
#[derive(Debug)]
pub struct BlockFrame<Ty> {
    block_ptr: usize,
    block_env: HashMap<Var, Ty>,
    lines: VecDeque<Var>,
}

impl<Ty> BlockFrame<Ty>
where
    Ty: Clone,
{
    fn get(&self, v: &Var) -> Option<Ty> {
        self.block_env.get(v).cloned()
    }
}

// This is the packaged up form of analysis
// which the interpreter returns after working
// on a particular method specialization.
#[derive(Debug)]
pub struct TypeAnalysis<Ty> {
    ts: Vec<Ty>,
}

#[derive(Debug)]
pub struct Interpreter<Ty, G> {
    pub sig: TypeSignature<Ty>,
    pub frame: BlockFrame<Ty>,
    block_queue: VecDeque<BlockFrame<Ty>>,
    pub state: InferenceState<Ty>,
    pub env: HashMap<Var, Ty>,
    global: Option<Arc<RwLock<G>>>,
}

impl<Ty, G> Interpreter<Ty, G>
where
    Ty: Clone,
{
    fn get(&self, v: &Var) -> Option<&Ty> {
        self.frame.block_env.get(v)
    }

    fn merge(&mut self) -> Result<(), TypeError> {
        for (k, v) in &self.frame.block_env {
            self.env.insert(*k, v.clone());
        }
        Ok(())
    }
}

/////
///// Type propagation API
/////

/*

   A type interpreter which utilizes forward propagation
   is defined by a unique state transition function (here, `propagate`)

   -- this method provides a way for the type interpreter to
   evaluate the effects of an IR instruction on the lattice defined
   by `Ty`.

*/

pub trait Propagation<A, Ty, E> {
    fn propagate<G>(interp: &mut Interpreter<Ty, G>, instr: &Instruction<Self, A>) -> Result<Ty, E>
    where
        Self: Sized;
}

/*

   The `Communication` trait provides a mechanism for type interpreters
   to synchronize across some global state `G`
   (as part of higher-level inference processes).

*/

pub trait Communication<M, R> {
    fn ask(&self, msg: &M) -> Option<R>;
}

impl<I, A, G, Ty> AbstractInterpreter<ExtIR<I, A>, TypeAnalysis<Ty>> for Interpreter<Ty, G>
where
    Ty: Clone,
    I: Propagation<A, Ty, TypeError> + Sized,
    G: Communication<TypeSignature<Ty>, Ty>,
{
    type LatticeElement = Ty;
    type Error = TypeError;
    type Meta = TypeSignature<Ty>;

    fn prepare(meta: Self::Meta, ir: &ExtIR<I, A>) -> Result<Interpreter<Ty, G>, Self::Error> {
        let mut initial_env: HashMap<Var, Ty> = HashMap::new();
        for (t, v) in meta.ts.iter().zip(ir.get_args()) {
            initial_env.insert(v, t.clone());
        }
        let bf = BlockFrame {
            block_ptr: 0,
            block_env: initial_env,
            lines: VecDeque::from(ir.get_vars_in_block(0)),
        };
        let vd = VecDeque::<BlockFrame<Ty>>::new();
        Ok(Interpreter {
            sig: meta,
            frame: bf,
            block_queue: vd,
            state: InferenceState::Active,
            env: HashMap::<Var, Ty>::new(),
            global: None,
        })
    }

    fn step(&mut self, ir: &ExtIR<I, A>) -> Result<(), Self::Error> {
        match &self.state {
            InferenceState::Waiting(tsig) => {
                // This should never panic.
                let v = self.frame.lines.pop_front().unwrap();
                match &self.global {
                    None => (),
                    Some(rr) => {
                        let read = rr.read().unwrap();
                        match read.ask(tsig) {
                            None => (),
                            Some(t) => {
                                self.frame.block_env.insert(v, t);
                                self.state = InferenceState::Active;
                            }
                        }
                    }
                }
            }

            InferenceState::Active => {
                let v = self.frame.lines.pop_front();
                match v {
                    None => {
                        self.merge();
                        match self.block_queue.pop_front() {
                            None => self.state = InferenceState::Finished,
                            Some(blk) => {
                                self.frame = blk;
                                self.state = InferenceState::Active;
                            }
                        }
                    }

                    Some(el) => match ir.get_instr(el) {
                        None => {
                            self.state = InferenceState::Error(TypeError::FailedToLookupVarInIR)
                        }
                        Some((v, instr)) => match &instr.op {
                            Operator::Intrinsic(_intr) => match Propagation::propagate(self, instr)
                            {
                                Ok(t) => {
                                    self.frame.block_env.insert(v, t);
                                    self.state = InferenceState::Active;
                                }
                                Err(e) => self.state = InferenceState::Error(e),
                            },
                            Operator::ModuleRef(_, n) => {
                                match instr
                                    .args
                                    .iter()
                                    .map(|x| self.frame.get(x))
                                    .collect::<Option<Vec<_>>>()
                                {
                                    None => {
                                        self.state = InferenceState::Error(
                                            TypeError::NotAllArgumentsTypedInCall,
                                        );
                                    }
                                    Some(ts) => {
                                        self.frame.lines.push_front(v);
                                        self.state = InferenceState::Waiting(TypeSignature {
                                            name: n.to_string(),
                                            ts,
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

    fn result(&mut self) -> Result<TypeAnalysis<Ty>, Self::Error> {
        let mut env = self.env.drain().collect::<Vec<_>>();
        env.sort_by(|a, b| a.0.id.partial_cmp(&b.0.id).unwrap());
        let ts = env.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
        let analysis = TypeAnalysis { ts };
        Ok(analysis)
    }
}
