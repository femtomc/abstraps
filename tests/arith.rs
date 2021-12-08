use abstraps::builder::ExtIRBuilder;
use abstraps::interp::{Communication, Interpreter, InterpreterError, Meta, Propagation};
use abstraps::ir::{AbstractInterpreter, Instruction, Operator};
use std::fmt;

// --------------- Arithmetic example --------------- //

// Fully specify the interpreter interface.

#[derive(Debug)]
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

#[derive(Debug)]
pub struct ArithAttribute;

impl fmt::Display for ArithAttribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithLattice {
    V(i64),
}

#[derive(Debug)]
struct Global;
impl Communication<Meta<ArithLattice>, ArithLattice> for Global {
    fn ask(&self, msg: &Meta<ArithLattice>) -> Option<ArithLattice> {
        return None;
    }
}

impl Propagation<ArithIntrinsic, ArithAttribute, ArithLattice, InterpreterError>
    for Interpreter<ArithIntrinsic, ArithAttribute, ArithLattice, Global>
{
    fn propagate(
        &mut self,
        instr: &Instruction<ArithIntrinsic, ArithAttribute>,
    ) -> Result<ArithLattice, InterpreterError> {
        let v = instr
            .get_args()
            .iter()
            .map(|v| match self.get(v) {
                None => Err(InterpreterError::FailedToLookupVarInEnv),
                Some(v) => match v {
                    ArithLattice::V(n) => Ok(n),
                },
            })
            .collect::<Result<Vec<_>, _>>()?;
        match instr.get_op() {
            Operator::Intrinsic(I) => match I {
                ArithIntrinsic::Add => Ok(ArithLattice::V(v.iter().sum())),
                ArithIntrinsic::Mul => Ok(ArithLattice::V(v.iter().fold(1, |a, b| a * b))),
            },
            Operator::ModuleRef(_, _) => Err(InterpreterError::Caseless),
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
    builder.build_instr(Operator::Intrinsic(ArithIntrinsic::Mul), args, Vec::new());
    let ir = builder.dump();
    println!("{}", ir);
    let m = Meta::new("".to_string(), vec![ArithLattice::V(6), ArithLattice::V(6)]);
    let mut interp =
        Interpreter::<ArithIntrinsic, ArithAttribute, ArithLattice, Global>::prepare(m, &ir)
            .unwrap();
    interp.step(&ir);
    interp.step(&ir);
    interp.step(&ir);
    interp.step(&ir);
    println!("{:?}", interp);
    let analysis = interp.result();
    println!("{:?}", analysis);
}
