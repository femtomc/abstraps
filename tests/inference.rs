use abstraps::builder::ExtIRBuilder;
use abstraps::interp::{
    Communication, Interpreter, InterpreterError, LatticeJoin, Meta, Propagation,
};
use abstraps::ir::{AbstractInterpreter, Instruction, Operator, Var};
use std::fmt;

// --------------- Smoke test --------------- //

// Fully specify the interpreter interface.

#[derive(Debug)]
pub enum Intrinsic0 {
    Fake,
}

impl fmt::Display for Intrinsic0 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "fake")
    }
}

#[derive(Debug)]
pub enum Attribute0 {
    Fake,
}

impl fmt::Display for Attribute0 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Fake")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lattice0 {
    Fake,
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LatticeError0 {
    Fake,
}

impl LatticeJoin<LatticeError0> for Lattice0 {
    fn join(&self, other: &Self) -> Result<Lattice0, LatticeError0> {
        Ok(self.clone())
    }
}

#[derive(Debug)]
struct Global;
impl Communication<Meta<Lattice0>, Lattice0> for Global {
    fn ask(&self, msg: &Meta<Lattice0>) -> Option<Lattice0> {
        return None;
    }
}

impl Propagation<Intrinsic0, Attribute0, Lattice0, LatticeError0>
    for Interpreter<Global, Intrinsic0, Attribute0, Lattice0, LatticeError0, Global>
{
    fn propagate(
        &mut self,
        curr: Var,
        instr: &Instruction<Intrinsic0, Attribute0>,
    ) -> Result<Lattice0, LatticeError0> {
        return Ok(Lattice0::Fake);
    }
}

#[test]
fn infer_0() {
    let mut builder = ExtIRBuilder::<Intrinsic0, Attribute0>::default();
    let v = builder.push_arg();
    builder.build_instr(Operator::Intrinsic(Intrinsic0::Fake), vec![v], Vec::new());
    let args = builder.jump_blk(vec![v]);
    builder.build_instr(Operator::Intrinsic(Intrinsic0::Fake), args, Vec::new());
    let ir = builder.dump();
    let m = Meta::new("".to_string(), vec![Lattice0::Fake]);
    let mut interp =
        Interpreter::<Global, Intrinsic0, Attribute0, Lattice0, LatticeError0, Global>::prepare(
            m, &ir,
        )
        .unwrap();
    interp.step(&ir);
    interp.step(&ir);
    interp.step(&ir);
    interp.step(&ir);
    let analysis = interp.finish();
}
