use std::fmt;
use yansi::Paint;

/// Types which are considered builtin to the framework.
///
/// Users who wish to extend this lattice can easily do so
/// by defining their own `enum` type.
///
/// In general, propagation rules are defined for most
/// of the standard dialects (e.g. [`crate::dialects::arith`], [`crate::dialects::memref`], etc).
#[derive(Debug)]
pub enum BuiltinLattice {
    Float32,
    Float64,
    Int32,
    Int64,
    MemRef(Box<BuiltinLattice>),
    Function(Vec<BuiltinLattice>, Box<BuiltinLattice>),
    Tensor(Vec<usize>, Box<BuiltinLattice>),
}

impl fmt::Display for BuiltinLattice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BuiltinLattice::Float32 => write!(f, "{}", Paint::magenta("Float32").bold()),
            BuiltinLattice::Float64 => write!(f, "{}", Paint::magenta("Float64").bold()),
            BuiltinLattice::Int32 => write!(f, "{}", Paint::magenta("Int32").bold()),
            BuiltinLattice::Int64 => write!(f, "{}", Paint::magenta("Int64").bold()),
            BuiltinLattice::MemRef(l) => write!(f, "{}<{}>", Paint::magenta("memref"), l),
            BuiltinLattice::Function(v, r) => {
                let l = v.len();
                write!(f, "<")?;
                for (ind, t) in v.iter().enumerate() {
                    match ind == l - 1 {
                        true => write!(f, "{}", t)?,
                        false => write!(f, "{}, ", t)?,
                    };
                }
                write!(f, "> -> {}", r)?;
                Ok(())
            }
            BuiltinLattice::Tensor(v, t) => {
                write!(f, "{}<{:?}, {}>", Paint::magenta("tensor"), v, t)
            }
        }
    }
}
