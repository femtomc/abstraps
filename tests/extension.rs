use abstraps::dialects::base::*;
use abstraps::dialects::builtin::*;
use abstraps::*;

// Declare `NonVariadic` as an extern interface,
// and provide the implementation.
//
// The `intrinsic!` macro otherwise just assumes that interface
// traits specified in the list before (extern: [...])
// have unital implementations.
intrinsic!(Add: ["arith", "add"], [], extern: [NonVariadic]);

impl NonVariadic for Add {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        if op.get_operands().len() != 2 {
            bail!(format!(
                "{} is non-variadic, and supports a fixed number (2) of operands.\n=> {}",
                op.get_intrinsic(),
                op
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

#[test]
fn extensions_0() -> Result<(), Report> {
    diagnostics_setup()?;
    let mut module = Module.get_builder("foo", LocationInfo::Unknown);
    let mut func = Func.get_builder("new_func", LocationInfo::Unknown);
    let operands = vec![func.push_arg()?, func.push_arg()?];
    let add1 = Add.get_builder(operands, LocationInfo::Unknown);
    let ret = func.push(add1)?;
    let add2 = Add.get_builder(vec![ret, ret], LocationInfo::Unknown);
    let v = func.push(add2)?;
    func.push(Return.get_builder(vec![v], LocationInfo::Unknown))?;
    module.push(func)?;
    let end = module.finish();
    assert!(end.is_ok());
    println!("{}", end.unwrap());
    Ok(())
}
