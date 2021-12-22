use abstraps::dialects::builtin::*;
use abstraps::*;
use color_eyre::Report;

#[derive(Clone, Debug)]
pub struct Add;

impl Intrinsic for Add {
    fn get_namespace(&self) -> &str {
        "arith"
    }

    fn get_name(&self) -> &str {
        "add"
    }
}

impl Add {
    pub fn get_builder(&self, operands: Vec<Var>, loc: LocationInfo) -> OperationBuilder {
        let intr = Box::new(Add);
        let mut b = OperationBuilder::default(intr, loc);
        b.set_operands(operands);
        b
    }
}

interfaces!(Add: dyn ObjectClone, dyn Intrinsic);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum ArithLattice {
    Int64,
}

impl std::fmt::Display for ArithLattice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ArithLattice::Int64 => write!(f, "Int64"),
        }
    }
}

impl LatticeJoin for ArithLattice {
    fn join(&self, _other: &ArithLattice) -> ArithLattice {
        self.clone()
    }
}

// Propagation rules.
impl LatticeSemantics<ArithLattice> for Add {
    fn propagate(&self, _v: Vec<&ArithLattice>) -> Result<ArithLattice, Report> {
        Ok(ArithLattice::Int64)
    }
}

#[test]
fn typeinf_0() -> Result<(), Report> {
    // When using the abstract interpretation system,
    // you must declare the propagation rule as a dynamic interface.
    dynamic_interfaces! {
        Add: dyn LatticeSemantics<ArithLattice>;
    }

    diagnostics_setup();
    let mut func1 = Func.get_builder("new_func1", LocationInfo::Unknown);
    let operands = vec![func1.push_arg()?, func1.push_arg()?];
    let add1 = Add.get_builder(operands, LocationInfo::Unknown);
    let ret = func1.push(add1)?;
    let add2 = Add.get_builder(vec![ret, ret], LocationInfo::Unknown);
    func1.push(add2)?;
    let end = func1.finish();
    assert!(end.is_ok());
    let op = end.unwrap();
    let key = Signature::new(
        "new_func1",
        vec![Some(ArithLattice::Int64), Some(ArithLattice::Int64)],
    );
    let mut am = AnalysisManager::new();
    am.analyze(key, &op)?;
    println!("{}", op);
    println!("{}", am);
    Ok(())
}
