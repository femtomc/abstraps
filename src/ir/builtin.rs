use crate::ir::builder::OperationBuilder;
use crate::ir::core::{BasicBlock, Intrinsic, IntrinsicTrait, Operation, Region, Var};
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub struct Module;

impl Intrinsic for Module {
    fn get_namespace(&self) -> &str {
        "builtin"
    }

    fn get_name(&self) -> &str {
        "module"
    }

    fn get_traits(&self) -> Vec<Box<dyn IntrinsicTrait>> {
        Vec::new()
    }

    fn get_builder(&self) -> OperationBuilder {
        let intr = Box::new(Module);
        OperationBuilder::default(intr)
    }
}

#[derive(Debug)]
pub struct Func;
impl Intrinsic for Func {
    fn get_namespace(&self) -> &str {
        "builtin"
    }

    fn get_name(&self) -> &str {
        "func"
    }

    fn get_traits(&self) -> Vec<Box<dyn IntrinsicTrait>> {
        Vec::new()
    }

    fn get_builder(&self) -> OperationBuilder {
        let intr = Box::new(Func);
        OperationBuilder::default(intr)
    }
}
