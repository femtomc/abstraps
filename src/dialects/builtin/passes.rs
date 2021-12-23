use crate::dialects::builtin::traits::{ProvidesSymbol, ProvidesSymbolTable};
use crate::*;
use std::sync::RwLock;
use yansi::Paint;

#[derive(Debug)]
pub struct PopulateSymbolTablePass;

impl OperationPass for PopulateSymbolTablePass {
    fn reset(&self) -> Box<dyn OperationPass> {
        Box::new(PopulateSymbolTablePass)
    }

    fn check(&self, op_lock: &RwLock<Operation>) -> Result<(), Report> {
        let op = &*op_lock.read().unwrap();
        let intr = op.get_intrinsic();
        match intr.query_ref::<dyn ProvidesSymbolTable>() {
            None => bail!(format!(
                "Operation does not satisfy the {} interface trait.",
                Paint::magenta("ProvidesSymbolTable").bold()
            )),
            Some(v) => v.verify(op)?,
        }
        Ok(())
    }

    fn apply(
        &self,
        op_lock: &RwLock<Operation>,
        _analysis_lock: &RwLock<AnalysisManager>,
    ) -> Result<(), Report> {
        let v = {
            let op = &*op_lock.read().unwrap();
            let region = &op.get_regions()[0];
            let mut v: Vec<(String, Var)> = Vec::new();
            for (var, child) in region.get_block_iter(0) {
                let intr = child.get_intrinsic();
                match intr.query_ref::<dyn ProvidesSymbol>() {
                    None => (),
                    Some(trt) => match trt.verify(op) {
                        Ok(_) => v.push((trt.get_value(child).to_string(), var)),
                        Err(_) => (),
                    },
                }
            }
            v
        };
        let mut op = op_lock.write().unwrap();
        let op_intr = op.get_intrinsic().clone();
        let tbl = op_intr
            .query_ref::<dyn ProvidesSymbolTable>()
            .unwrap()
            .get_value_mut(&mut *op);
        for (s, v) in v.into_iter() {
            tbl.insert(s, v);
        }
        Ok(())
    }
}
