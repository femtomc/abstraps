use abstraps::dialects::builtin::*;
use abstraps::*;
use color_eyre::Report;

#[derive(Debug, Clone)]
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

interfaces!(Add: dyn Intrinsic);

#[test]
fn extensions_0() -> Result<(), Report> {
    let mut module = Module.get_builder("foo", LocationInfo::Unknown);
    let mut func = Func.get_builder("new_func", LocationInfo::Unknown);
    let operands = vec![func.push_arg()?, func.push_arg()?];
    let add1 = Add.get_builder(operands, LocationInfo::Unknown);
    let ret = func.push(add1)?;
    let add2 = Add.get_builder(vec![ret, ret], LocationInfo::Unknown);
    func.push(add2)?;
    module.push(func)?;
    let end = module.finish();
    assert!(end.is_ok());
    println!("{}", end.unwrap());
    Ok(())
}
