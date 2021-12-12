use abstraps;
use abstraps::ir::builder::{OperationBuilder, Push, Setup};
use abstraps::ir::builtin::{BuiltinAttribute, BuiltinIntrinsic, Symbol, SymbolTable};
use abstraps::ir::core::{Operation, Var, Verify};
use derive_more::{Display, From, Into};
use serde::Serialize;
use std::collections::HashMap;

// This is an example of how `bifmap` (a bifunctor-like mapping)
// supports intrinsic/attribute extension mechanism
// (so e.g. the user can combine sets of intrinsics/attributes
// build them using the functionality defined for sub-sets,
// and then convert them to the toplevel dialect).

#[derive(Debug, Display, From)]
enum ExtendedIntrinsic {
    B(BuiltinIntrinsic),
    Add,
    Mul,
}

#[derive(Debug, Display, From)]
enum ExtendedAttribute {
    B(BuiltinAttribute),
    Constant(i64),
}

impl SymbolTable for ExtendedAttribute {
    fn get_table(&mut self) -> Option<&mut HashMap<String, Var>> {
        match self {
            ExtendedAttribute::B(bi) => bi.get_table(),
            _ => None,
        }
    }
}

impl Symbol for ExtendedAttribute {
    fn get_symbol(&self) -> Option<String> {
        match self {
            ExtendedAttribute::B(bi) => bi.get_symbol(),
            _ => None,
        }
    }
}

#[test]
fn extension_module_operation_0() {
    let mut builder =
        OperationBuilder::<BuiltinIntrinsic, BuiltinAttribute>::new(BuiltinIntrinsic::Module);
    let op = builder.finish();
    // At this stage,
    // the operation is <BuiltinIntrinsic, BuiltinAttribute>.
    assert!(op.is_ok());

    // The `bifmap` interface allows recursive conversion throughout
    // the IR.
    let new = op.unwrap().bifmap(&|x| ExtendedIntrinsic::from(x), &|x| {
        ExtendedAttribute::from(x)
    });
}

#[test]
fn extension_module_operation_1() {
    let mut module =
        OperationBuilder::<BuiltinIntrinsic, BuiltinAttribute>::new(BuiltinIntrinsic::Module)
            .name("foo");
    let mut func =
        OperationBuilder::<BuiltinIntrinsic, BuiltinAttribute>::new(BuiltinIntrinsic::Func)
            .name("new_func");
    let mut add =
        OperationBuilder::<ExtendedIntrinsic, ExtendedAttribute>::default(ExtendedIntrinsic::Add);
    let operands = vec![func.push_arg().unwrap(), func.push_arg().unwrap()];
    add.set_operands(operands);
    let op = func.push_op(add.finish().unwrap()).finish().unwrap();
    println!("{}", module.push_op(op).finish().unwrap());
}
