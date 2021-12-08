use abstraps::builder::ExtIRBuilder;
use abstraps::interp::{Communication, Interpreter, InterpreterError, Meta, Propagation};
use abstraps::ir::{AbstractInterpreter, Instruction, Operator};
use std::fmt;

// --------------- SMOKE TEST 0 --------------- //

// Fully specify the interpreter interface.

#[derive(Debug)]
pub enum FakeIntrinsic {
    Fake,
}

impl fmt::Display for FakeIntrinsic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "fake")
    }
}

#[derive(Debug)]
pub enum FakeAttribute {
    Fake,
}

impl fmt::Display for FakeAttribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Fake")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FakeLattice {
    Fake,
    Other(String),
}

#[derive(Debug)]
struct Global;
impl Communication<Meta<FakeLattice>, FakeLattice> for Global {
    fn ask(&self, msg: &Meta<FakeLattice>) -> Option<FakeLattice> {
        return None;
    }
}

impl Propagation<FakeIntrinsic, FakeAttribute, FakeLattice, InterpreterError>
    for Interpreter<FakeIntrinsic, FakeAttribute, FakeLattice, Global>
{
    fn propagate(
        &mut self,
        instr: &Instruction<FakeIntrinsic, FakeAttribute>,
    ) -> Result<FakeLattice, InterpreterError> {
        return Ok(FakeLattice::Fake);
    }
}

#[test]
fn infer_0() {
    let mut builder = ExtIRBuilder::<FakeIntrinsic, FakeAttribute>::default();
    let v = builder.push_arg();
    builder.build_instr(
        Operator::Intrinsic(FakeIntrinsic::Fake),
        vec![v],
        Vec::new(),
    );
    let args = builder.jump_blk(vec![v]);
    builder.build_instr(Operator::Intrinsic(FakeIntrinsic::Fake), args, Vec::new());
    let ir = builder.dump();
    println!("{}", ir);
    let m = Meta::new("".to_string(), vec![FakeLattice::Fake]);
    let mut interp =
        Interpreter::<FakeIntrinsic, FakeAttribute, FakeLattice, Global>::prepare(m, &ir).unwrap();
    interp.step(&ir);
    interp.step(&ir);
    interp.step(&ir);
    interp.step(&ir);
    println!("{:?}", interp);
    let analysis = interp.result();
    println!("{:?}", analysis);
}

// --------------- SMOKE TEST 1 --------------- //
