use abstraps::builder::ExtIRBuilder;
use abstraps::interp::{Communication, Interpreter, LatticeJoin, Meta, Propagation};
use abstraps::ir::{AbstractInterpreter, Instruction, Operator, Var};
use std::collections::{HashMap, HashSet};
use std::fmt;

// While testing: to see stdout, run with `cargo test -- --nocapture`.

// --------------- Arithmetic example --------------- //

// Fully specify the interpreter interface.

#[derive(Debug, Clone)]
pub enum ArithIntrinsic {
    Add,
    Mul,
}

impl fmt::Display for ArithIntrinsic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArithIntrinsic::Add => write!(f, "add"),
            ArithIntrinsic::Mul => write!(f, "mul"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArithAttribute;

impl fmt::Display for ArithAttribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithLattice {
    Int,
    Float,
}

impl fmt::Display for ArithLattice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArithLattice::Int => write!(f, "Int"),
            ArithLattice::Float => write!(f, "Float"),
        }
    }
}

impl LatticeJoin<LatticeError> for ArithLattice {
    // This automatically supports widening to Float.
    fn join(&self, other: &Self) -> Result<ArithLattice, LatticeError> {
        match self {
            ArithLattice::Float => Ok(ArithLattice::Float),
            ArithLattice::Int => match other {
                ArithLattice::Float => Ok(ArithLattice::Float),
                ArithLattice::Int => Ok(ArithLattice::Int),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LatticeError {
    Fallback,
}

#[derive(Debug, Clone)]
struct Global;
impl Communication<Meta<ArithLattice>, ArithLattice> for Global {
    fn ask(&self, msg: &Meta<ArithLattice>) -> Option<ArithLattice> {
        return None;
    }
}

impl Propagation<ArithIntrinsic, ArithAttribute, ArithLattice, LatticeError>
    for Interpreter<Global, ArithIntrinsic, ArithAttribute, ArithLattice, LatticeError, Global>
{
    fn propagate(
        &mut self,
        curr: Var,
        instr: &Instruction<ArithIntrinsic, ArithAttribute>,
    ) -> Result<ArithLattice, LatticeError> {
        let v = instr
            .get_args()
            .iter()
            .map(|v| match self.get(v) {
                None => Err(LatticeError::Fallback),
                Some(v) => Ok(v),
            })
            .collect::<Result<Vec<_>, _>>()?;
        match instr.get_op() {
            Operator::Intrinsic(I) => match I {
                ArithIntrinsic::Add => {
                    Ok(v.iter().try_fold(ArithLattice::Int, |a, b| a.join(b))?)
                }
                ArithIntrinsic::Mul => {
                    Ok(v.iter().try_fold(ArithLattice::Int, |a, b| a.join(b))?)
                }
            },
            Operator::ModuleRef(_, _) => Err(LatticeError::Fallback),
        }
    }
}

#[test]
fn arith_0() {
    let mut builder = ExtIRBuilder::<ArithIntrinsic, ArithAttribute>::default();
    let v1 = builder.push_arg();
    let v2 = builder.push_arg();
    let v3 = builder.build_instr(
        Operator::Intrinsic(ArithIntrinsic::Add),
        vec![v1, v2],
        Vec::new(),
    );
    let args = builder.jump_blk(vec![v1, v2, v3]);
    let v4 = builder.build_instr(Operator::Intrinsic(ArithIntrinsic::Mul), args, Vec::new());
    builder.push_branch(None, builder.get_block_ptr(), vec![v4, v4, v4]);
    let ir = builder.dump();
    let m = Meta::new("".to_string(), vec![ArithLattice::Int, ArithLattice::Float]);
    let mut interp = Interpreter::<
        Global,
        ArithIntrinsic,
        ArithAttribute,
        ArithLattice,
        LatticeError,
        Global,
    >::prepare(m, &ir)
    .unwrap();
    interp.step(&ir);
    interp.step(&ir);
    interp.step(&ir);
    interp.step(&ir);
    interp.step(&ir);
    interp.step(&ir);
    let analysis = interp.get_intermediate_frame().unwrap();
    println!("{}", ir);
    println!("{:?}", interp);
    println!("{:?}", analysis);
    let v = analysis.get_ret().unwrap();
}
