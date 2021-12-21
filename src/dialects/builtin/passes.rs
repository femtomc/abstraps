use crate::core::{
    AnalysisManager, IntrinsicTrait, Operation, OperationPass, SupportsVerification, Var,
};
use crate::dialects::builtin::attributes::{Symbol, SymbolTable};

use crate::dialects::builtin::traits::{ProvidesSymbol, ProvidesSymbolTable};
use color_eyre::Report;

use std::sync::RwLock;

#[derive(Debug)]
pub struct PopulateSymbolTablePass;

impl OperationPass for PopulateSymbolTablePass {
    fn reset(&self) -> Box<dyn OperationPass> {
        Box::new(PopulateSymbolTablePass)
    }

    fn apply(
        &self,
        op_lock: &RwLock<Operation>,
        _analysis_lock: &RwLock<AnalysisManager>,
    ) -> Result<(), Report> {
        let mut op = op_lock.write().unwrap();
        let tr = op.get_trait::<ProvidesSymbolTable>()?;
        let region = &op.get_regions()[0];
        let mut v: Vec<(String, Var)> = Vec::new();
        for (var, child) in region.get_block_iter(0) {
            if child.has_trait::<ProvidesSymbol>() {
                let s_tr = child.get_trait::<ProvidesSymbol>()?;
                let s_attr = s_tr.get_attribute(child)?;
                let s = s_attr.downcast_ref::<Symbol>().unwrap();
                v.push((s.to_string(), var));
            }
        }
        let attr = tr.get_attribute_mut(&mut op)?;
        let tbl = attr.downcast_mut::<SymbolTable>().unwrap();
        for (s, v) in v.iter() {
            tbl.insert(s, *v);
        }
        Ok(())
    }
}
