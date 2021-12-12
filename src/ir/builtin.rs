use crate::ir::builder::{OperationBuilder, Setup};
use crate::ir::core::{BasicBlock, Operation, Region, Var, Verify};
use crate::ir::graph::Graph;
use crate::ir::ssacfg::SSACFG;
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum BuiltinIntrinsic {
    Module,
    Func,
    Call,
    Branch,
    ConditionalBranch,
}

impl fmt::Display for BuiltinIntrinsic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BuiltinIntrinsic::Module => write!(f, "module"),
            BuiltinIntrinsic::Func => write!(f, "func"),
            BuiltinIntrinsic::Call => write!(f, "call"),
            BuiltinIntrinsic::Branch => write!(f, "br"),
            BuiltinIntrinsic::ConditionalBranch => write!(f, "condbr"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BuiltinAttribute {
    Symbol(String),
    SymbolTable(HashMap<String, Var>),
}

impl fmt::Display for BuiltinAttribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BuiltinAttribute::SymbolTable(m) => {
                write!(f, "{:?}", m)
            }
            BuiltinAttribute::Symbol(s) => {
                write!(f, "{}", s)
            }
        }
    }
}

impl Setup<BuiltinIntrinsic, BuiltinAttribute>
    for OperationBuilder<BuiltinIntrinsic, BuiltinAttribute>
{
    fn new(intr: BuiltinIntrinsic) -> OperationBuilder<BuiltinIntrinsic, BuiltinAttribute> {
        match intr {
            BuiltinIntrinsic::Module => {
                let mut b = OperationBuilder::<BuiltinIntrinsic, BuiltinAttribute>::default(intr);
                let r = Region::Undirected(Graph::default());
                b.push_region(r);
                let blk = BasicBlock::<BuiltinIntrinsic, BuiltinAttribute>::default();
                b.push_block(blk);
                let st = BuiltinAttribute::SymbolTable(HashMap::new());
                b.insert_attr("symbols", st);
                b
            }

            BuiltinIntrinsic::Func => {
                let mut b = OperationBuilder::<BuiltinIntrinsic, BuiltinAttribute>::default(intr);
                let r = Region::Directed(SSACFG::default());
                b.push_region(r);
                let blk = BasicBlock::<BuiltinIntrinsic, BuiltinAttribute>::default();
                b.push_block(blk);
                b
            }

            BuiltinIntrinsic::Call => {
                OperationBuilder::<BuiltinIntrinsic, BuiltinAttribute>::default(intr)
            }

            BuiltinIntrinsic::Branch => {
                let mut b = OperationBuilder::<BuiltinIntrinsic, BuiltinAttribute>::default(intr);
                b
            }

            BuiltinIntrinsic::ConditionalBranch => {
                let mut b = OperationBuilder::<BuiltinIntrinsic, BuiltinAttribute>::default(intr);
                b
            }
        }
    }
}

impl OperationBuilder<BuiltinIntrinsic, BuiltinAttribute> {
    pub fn name(mut self, name: &str) -> Self {
        let attr = BuiltinAttribute::Symbol(name.to_string());
        self.insert_attr("symbol", attr);
        self
    }
}

pub trait Symbol {
    fn get_symbol(&self) -> Option<String>;
}

impl Symbol for BuiltinAttribute {
    fn get_symbol(&self) -> Option<String> {
        match self {
            BuiltinAttribute::Symbol(s) => Some(s.to_string()),
            BuiltinAttribute::SymbolTable(t) => None,
        }
    }
}

pub trait SymbolTable {
    fn get_table(&mut self) -> Option<&mut HashMap<String, Var>>;
}

impl SymbolTable for BuiltinAttribute {
    fn get_table(&mut self) -> Option<&mut HashMap<String, Var>> {
        match self {
            BuiltinAttribute::Symbol(s) => None,
            BuiltinAttribute::SymbolTable(t) => Some(t),
        }
    }
}
