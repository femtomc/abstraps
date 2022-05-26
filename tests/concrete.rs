use abstraps::core::*;
use abstraps::dialects::arith::*;
use abstraps::dialects::base::*;
use abstraps::dialects::builtin::*;
use abstraps::*;

// This represents a partial concrete / partial abstract lattice.
// E.g. as might be used inside a partial evaluator.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum ArithLattice {
    Bottom,
    Int64,
    Float64,
    Value(i64),
    Union(Vec<ArithLattice>),
}

impl std::fmt::Display for ArithLattice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ArithLattice::Bottom => write!(f, "{{}}"),
            ArithLattice::Int64 => write!(f, "Int64"),
            ArithLattice::Float64 => write!(f, "Float64"),
            ArithLattice::Value(v) => write!(f, "Val({})", v),
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

// This is an interface which allows widening.
// As required for infinite lattices, etc.
impl LatticeJoin for ArithLattice {
    fn join(&self, other: &ArithLattice) -> ArithLattice {
        match (self, other) {
            (ArithLattice::Bottom, v) | (v, ArithLattice::Bottom) => v.clone(),
            (ArithLattice::Int64, ArithLattice::Float64)
            | (ArithLattice::Float64, ArithLattice::Int64) => {
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
            (ArithLattice::Value(v1), ArithLattice::Value(v2)) => ArithLattice::Int64,
            (_, _) => self.clone(),
        }
    }
}

// Propagation rules.
// TODO: actually get other propagated types from IR registers.
// Instead of just saying "oh, it's an Int".
impl LatticeSemantics<ArithLattice> for Addi {
    fn propagate(
        &self,
        _op: &Operation,
        vtypes: Vec<&ArithLattice>,
    ) -> Result<ArithLattice, Report> {
        match vtypes[..] {
            [ArithLattice::Int64, ArithLattice::Int64] => Ok(ArithLattice::Int64),
            [ArithLattice::Float64, ArithLattice::Int64]
            | [ArithLattice::Int64, ArithLattice::Float64] => Ok(ArithLattice::Float64),
            [ArithLattice::Float64, ArithLattice::Float64] => Ok(ArithLattice::Float64),
            [ArithLattice::Value(v1), ArithLattice::Value(v2)] => Ok(ArithLattice::Value(v1 + v2)),
            _ => bail!("vtypes is either empty, or no rule!"),
        }
    }
}

impl LatticeSemantics<ArithLattice> for Return {
    fn propagate(
        &self,
        _op: &Operation,
        vtypes: Vec<&ArithLattice>,
    ) -> Result<ArithLattice, Report> {
        match vtypes[..] {
            [v] => Ok(v.clone()),
            _ => bail!("Should be unreachable."),
        }
    }
}

#[test]
fn concrete_0() -> Result<(), Report> {
    diagnostics_setup()?;
    // When using the abstract interpretation system,
    // you must declare the propagation rule as a dynamic interface.
    dynamic_interfaces! {
        Return: dyn LatticeSemantics<ArithLattice>;
                Addi: dyn LatticeSemantics<ArithLattice>;
    }

    let mut func1 = Func.get_builder("new_func1", LocationInfo::Unknown)?;
    let operands = vec![func1.push_arg()?, func1.push_arg()?];
    let add1 = Addi.get_builder(operands, LocationInfo::Unknown)?;
    let ret = func1.push(add1)?;
    let add2 = Addi.get_builder(vec![ret, ret], LocationInfo::Unknown)?;
    let v = func1.push(add2)?;
    func1.push(Return.get_builder(vec![v], LocationInfo::Unknown)?)?;
    let end = func1.finish();
    assert!(end.is_ok());
    let op = end.unwrap();
    let key1 = Signature::new(
        "new_func1",
        vec![Some(ArithLattice::Value(2)), Some(ArithLattice::Value(3))],
    );
    let mut am = AnalysisManager::new();
    am.analyze(key1, &op)?;
    println!("{}", am);
    Ok(())
}
