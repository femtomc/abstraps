use abstraps::dialects::base::*;
use abstraps::dialects::builtin::*;
use abstraps::*;

intrinsic!(Add: ["arith", "add"], [], extern: [NonVariadic]);

impl NonVariadic for Add {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        if op.get_operands().len() != 2 {
            bail!(format!(
                "{} is non-variadic, and supports a fixed number (2) of operands.",
                op.get_intrinsic(),
            ));
        }
        Ok(())
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum ArithLattice {
    Bottom,
    Int64,
    Float64,
    Union(Vec<ArithLattice>),
}

impl std::fmt::Display for ArithLattice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ArithLattice::Bottom => write!(f, "{{}}"),
            ArithLattice::Int64 => write!(f, "Int64"),
            ArithLattice::Float64 => write!(f, "Float64"),
            ArithLattice::Union(v) => match v.is_empty() {
                true => write!(f, "Union{{}}"),
                false => {
                    let l = v.len();
                    for (ind, t) in v.iter().enumerate() {
                        match ind == l - 1 {
                            true => write!(f, "{}", t)?,
                            false => write!(f, "{}, ", t)?,
                        };
                    }
                    Ok(())
                }
            },
        }
    }
}

impl LatticeJoin for ArithLattice {
    fn join(&self, other: &ArithLattice) -> ArithLattice {
        match (self, other) {
            (ArithLattice::Bottom, v) => v.clone(),
            (v, ArithLattice::Bottom) => v.clone(),
            (ArithLattice::Int64, ArithLattice::Float64) => {
                ArithLattice::Union(vec![self.clone(), other.clone()])
            }
            (ArithLattice::Float64, ArithLattice::Int64) => {
                ArithLattice::Union(vec![other.clone(), self.clone()])
            }
            (ArithLattice::Union(v), ArithLattice::Float64) => {
                if v.contains(&ArithLattice::Float64) {
                    self.clone()
                } else {
                    let mut new = v.to_vec();
                    new.push(ArithLattice::Float64);
                    ArithLattice::Union(new)
                }
            }
            (_, _) => self.clone(),
        }
    }
}

// Propagation rules.
impl LatticeSemantics<ArithLattice> for Add {
    fn propagate(
        &self,
        _interp: &mut Interpreter<ArithLattice>,
        _op: &Operation,
    ) -> Result<ArithLattice, Report> {
        Ok(ArithLattice::Int64)
    }
}

// Propagation rules.
impl LatticeSemantics<ArithLattice> for Return {
    fn propagate(
        &self,
        _interp: &mut Interpreter<ArithLattice>,
        _op: &Operation,
    ) -> Result<ArithLattice, Report> {
        Ok(ArithLattice::Int64)
    }
}

#[test]
fn typeinf_0() -> Result<(), Report> {
    diagnostics_setup();
    // When using the abstract interpretation system,
    // you must declare the propagation rule as a dynamic interface.
    dynamic_interfaces! {
        Return: dyn LatticeSemantics<ArithLattice>;
        Add: dyn LatticeSemantics<ArithLattice>;
    }

    let mut func1 = Func.get_builder("new_func1", LocationInfo::Unknown);
    let operands = vec![func1.push_arg()?, func1.push_arg()?];
    let add1 = Add.get_builder(operands, LocationInfo::Unknown);
    let ret = func1.push(add1)?;
    let add2 = Add.get_builder(vec![ret, ret], LocationInfo::Unknown);
    let v = func1.push(add2)?;
    func1.push(Return.get_builder(vec![v], LocationInfo::Unknown))?;
    let end = func1.finish();
    assert!(end.is_ok());
    let op = end.unwrap();
    let key1 = Signature::new(
        "new_func1",
        vec![Some(ArithLattice::Int64), Some(ArithLattice::Int64)],
    );
    let mut am = AnalysisManager::new();
    am.analyze(key1, &op)?;
    let key2 = Signature::new(
        "new_func1",
        vec![Some(ArithLattice::Float64), Some(ArithLattice::Int64)],
    );
    am.analyze(key2, &op)?;
    println!("{}", op);
    println!("{}", am);
    Ok(())
}
